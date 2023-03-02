// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crypto::keys::slip10::Chain;
use instant::Instant;
use iota_client::{
    api_types::core::response::OutputWithMetadataResponse,
    block::{
        input::Input,
        output::{dto::OutputDto, Output, OutputId},
        payload::{
            transaction::{TransactionEssence, TransactionId},
            Payload, TransactionPayload,
        },
    },
    Client,
};

use crate::{
    account::{
        build_transaction_from_payload_and_inputs, handle::AccountHandle, types::OutputData, AddressWithUnspentOutputs,
    },
    task,
};

impl AccountHandle {
    /// Convert OutputWithMetadataResponse to OutputData with the network_id added
    pub(crate) async fn output_response_to_output_data(
        &self,
        output_responses: Vec<OutputWithMetadataResponse>,
        associated_address: &AddressWithUnspentOutputs,
    ) -> crate::Result<Vec<OutputData>> {
        log::debug!("[SYNC] convert output_responses");
        // store outputs with network_id
        let network_id = self.client.get_network_id().await?;
        let mut outputs = Vec::new();
        let token_supply = self.client.get_token_supply().await?;
        let account = self.read().await;

        #[allow(clippy::significant_drop_in_scrutinee)]
        for output_response in output_responses {
            let output = Output::try_from_dto(&output_response.output, token_supply)?;
            let transaction_id = TransactionId::from_str(&output_response.metadata.transaction_id)?;
            // check if we know the transaction that created this output and if we created it (if we store incoming
            // transactions separated, then this check wouldn't be required)
            let remainder = account
                .transactions
                .get(&transaction_id)
                .map_or(false, |tx| !tx.incoming);

            // 44 is for BIP 44 (HD wallets) and 4218 is the registered index for IOTA https://github.com/satoshilabs/slips/blob/master/slip-0044.md
            let chain = Chain::from_u32_hardened(vec![
                44,
                account.coin_type,
                account.index,
                associated_address.internal as u32,
                associated_address.key_index,
            ]);

            outputs.push(OutputData {
                output_id: OutputId::new(transaction_id, output_response.metadata.output_index)?,
                metadata: output_response.metadata.clone(),
                output,
                is_spent: output_response.metadata.is_spent,
                address: associated_address.address.inner,
                network_id,
                remainder,
                chain: Some(chain),
            });
        }

        Ok(outputs)
    }

    /// Gets outputs by their id, already known outputs are not requested again, but loaded from the account set as
    /// unspent, because we wouldn't get them from the node if they were spent
    pub(crate) async fn get_outputs(
        &self,
        output_ids: Vec<OutputId>,
    ) -> crate::Result<Vec<OutputWithMetadataResponse>> {
        log::debug!("[SYNC] start get_outputs");
        let get_outputs_start_time = Instant::now();
        let mut outputs = Vec::new();
        let mut unknown_outputs = Vec::new();
        let mut unspent_outputs = Vec::new();
        let mut account = self.write().await;

        #[allow(clippy::significant_drop_in_scrutinee)]
        for output_id in output_ids {
            match account.outputs.get_mut(&output_id) {
                // set unspent
                Some(output_data) => {
                    output_data.is_spent = false;
                    unspent_outputs.push((output_id, output_data.clone()));
                    outputs.push(OutputWithMetadataResponse {
                        metadata: output_data.metadata.clone(),
                        output: OutputDto::from(&output_data.output),
                    });
                }
                None => unknown_outputs.push(output_id),
            }
        }
        // known output is unspent, so insert it to the unspent outputs again, because if it was an
        // alias/nft/foundry output it could have been removed when syncing without them
        for (output_id, output_data) in unspent_outputs {
            account.unspent_outputs.insert(output_id, output_data);
        }

        drop(account);

        if !unknown_outputs.is_empty() {
            outputs.extend(self.client.get_outputs(unknown_outputs).await?);
        }

        log::debug!(
            "[SYNC] finished get_outputs in {:.2?}",
            get_outputs_start_time.elapsed()
        );

        Ok(outputs)
    }

    // Try to get transactions and inputs for received outputs
    // Because the transactions and outputs are pruned, we might can not get them anymore, in that case errors are not
    // returned
    pub(crate) async fn request_incoming_transaction_data(
        &self,
        transaction_ids: Vec<TransactionId>,
    ) -> crate::Result<()> {
        log::debug!("[SYNC] request_incoming_transaction_data");

        // Limit parallel requests to 100, to avoid timeouts
        for transaction_ids_chunk in transaction_ids.chunks(100).map(|x: &[TransactionId]| x.to_vec()) {
            let mut tasks = Vec::new();
            let account = self.read().await;

            for transaction_id in transaction_ids_chunk {
                // Don't request known or inaccessible transactions again
                if account.transactions.contains_key(&transaction_id)
                    || account.incoming_transactions.contains_key(&transaction_id)
                    || account.inaccessible_incoming_transactions.contains(&transaction_id)
                {
                    continue;
                }

                let client = self.client.clone();
                tasks.push(async move {
                    task::spawn(async move {
                        match client.get_included_block(&transaction_id).await {
                            Ok(block) => {
                                if let Some(Payload::Transaction(transaction_payload)) = block.payload() {
                                    let inputs =
                                        get_inputs_for_transaction_payload(&client, transaction_payload).await?;

                                    let transaction = build_transaction_from_payload_and_inputs(
                                        transaction_id,
                                        *transaction_payload.clone(),
                                        inputs,
                                    )?;

                                    Ok((transaction_id, Some(transaction)))
                                } else {
                                    Ok((transaction_id, None))
                                }
                            }
                            Err(iota_client::Error::NotFound(_)) => Ok((transaction_id, None)),
                            Err(e) => Err(crate::Error::Client(e.into())),
                        }
                    })
                    .await
                });
            }

            drop(account);

            let results = futures::future::try_join_all(tasks).await?;
            // Update account with new transactions
            let mut account = self.write().await;
            for res in results {
                match res? {
                    (transaction_id, Some(transaction)) => {
                        account.incoming_transactions.insert(transaction_id, transaction);
                    }
                    (transaction_id, None) => {
                        log::debug!("[SYNC] adding {transaction_id} to inaccessible_incoming_transactions");
                        // Save transactions that weren't found by the node to avoid requesting them endlessly.
                        // Will be cleared when new client options are provided.
                        account.inaccessible_incoming_transactions.insert(transaction_id);
                    }
                }
            }
        }

        Ok(())
    }
}

// Try to fetch the inputs of the transaction
pub(crate) async fn get_inputs_for_transaction_payload(
    client: &Client,
    transaction_payload: &TransactionPayload,
) -> crate::Result<Vec<OutputWithMetadataResponse>> {
    let TransactionEssence::Regular(essence) = transaction_payload.essence();
    let mut output_ids = Vec::new();

    for input in essence.inputs() {
        if let Input::Utxo(input) = input {
            output_ids.push(*input.output_id());
        }
    }

    client.try_get_outputs(output_ids).await.map_err(|e| e.into())
}
