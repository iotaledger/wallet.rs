// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{str::FromStr, time::Instant};

use crypto::keys::slip10::Chain;
use iota_client::{
    api::ClientBlockBuilder,
    bee_block::{
        output::{dto::OutputDto, Output, OutputId},
        payload::transaction::TransactionId,
    },
    bee_rest_api::types::responses::OutputResponse,
};

use crate::account::{handle::AccountHandle, types::OutputData, AddressWithUnspentOutputs};

impl AccountHandle {
    /// Convert OutputResponse to OutputData with the network_id added
    pub(crate) async fn output_response_to_output_data(
        &self,
        output_responses: Vec<OutputResponse>,
        associated_address: &AddressWithUnspentOutputs,
    ) -> crate::Result<Vec<OutputData>> {
        log::debug!("[SYNC] convert output_responses");
        // store outputs with network_id
        let account = self.read().await;
        let network_id = self.client.get_network_id().await?;
        let mut outputs = Vec::new();
        for output_response in output_responses {
            let (amount, address) = ClientBlockBuilder::get_output_amount_and_address(&output_response.output, None)?;
            let transaction_id = TransactionId::from_str(&output_response.metadata.transaction_id)?;
            // check if we know the transaction that created this output and if we created it (if we store incoming
            // transactions separated, then this check wouldn't be required)
            let remainder = {
                match account.transactions.get(&transaction_id) {
                    Some(tx) => !tx.incoming,
                    None => false,
                }
            };

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
                output: Output::try_from(&output_response.output)?,
                amount,
                is_spent: output_response.metadata.is_spent,
                address,
                network_id,
                remainder,
                chain: Some(chain),
            });
        }
        Ok(outputs)
    }

    /// Gets outputs by their id, already known outputs are not requested again, but loaded from the account set as
    /// unspent, because we wouldn't get them from the node if they were spent
    /// New requested output responses are returned and old ones separated with their accumulated balance
    pub(crate) async fn get_outputs(
        &self,
        output_ids: Vec<OutputId>,
        spent_outputs: bool,
    ) -> crate::Result<(Vec<OutputResponse>, u64, Vec<OutputResponse>)> {
        log::debug!("[SYNC] start get_outputs");
        let get_outputs_start_time = Instant::now();
        let mut found_outputs = Vec::new();
        let mut balance_from_known_outputs = 0;
        let mut loaded_outputs = Vec::new();
        // For spent outputs we want to try to fetch all, so we can update them locally
        if spent_outputs {
            found_outputs = self.client.try_get_outputs(output_ids).await?;
        } else {
            let mut unknown_outputs = Vec::new();
            let mut account = self.write().await;
            let mut unspent_outputs = Vec::new();
            for output_id in output_ids {
                match account.outputs.get_mut(&output_id) {
                    // set unspent
                    Some(output_data) => {
                        output_data.is_spent = false;
                        unspent_outputs.push((output_id, output_data.clone()));
                        loaded_outputs.push(OutputResponse {
                            metadata: output_data.metadata.clone(),
                            output: OutputDto::from(&output_data.output),
                        });
                        balance_from_known_outputs += output_data.amount
                    }
                    None => unknown_outputs.push(output_id),
                }
            }
            // known output is unspent, so insert it to the unspent outputs again, because if it was an
            // alias/nft/foundry output it could have been removed when syncing without `sync_aliases_and_nfts`
            for (output_id, output_data) in unspent_outputs {
                account.unspent_outputs.insert(output_id, output_data);
            }

            if !unknown_outputs.is_empty() {
                found_outputs = self.client.get_outputs(unknown_outputs).await?;
            }
        }

        log::debug!(
            "[SYNC] finished get_outputs in {:.2?}",
            get_outputs_start_time.elapsed()
        );
        Ok((found_outputs, balance_from_known_outputs, loaded_outputs))
    }
}
