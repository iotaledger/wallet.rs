// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    constants::PARALLEL_REQUESTS_AMOUNT,
    handle::AccountHandle,
    operations::syncing::SyncOptions,
    types::{OutputData, OutputKind},
};

use iota_client::{
    bee_message::{
        address::{Address, Ed25519Address},
        input::UtxoInput,
        output::OutputId,
        payload::transaction::TransactionId,
        MessageId,
    },
    bee_rest_api::types::{
        dtos::{AddressDto, OutputDto},
        responses::OutputResponse,
    },
};

use std::{
    str::FromStr,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

/// Convert OutputResponse to OutputData with the network_id added
pub(crate) async fn output_response_to_output_data(
    account_handle: &AccountHandle,
    output_responses: Vec<OutputResponse>,
) -> crate::Result<Vec<OutputData>> {
    log::debug!("[SYNC] convert output_responses");
    // store outputs with network_id
    let account = account_handle.read().await;
    let client = crate::client::get_client().await?;
    let network_id = client.get_network_id().await?;
    let bech32_hrp = client.get_bech32_hrp().await?;
    output_responses
        .into_iter()
        .map(|output| {
            let (amount, address, output_kind) = get_output_amount_and_address(&output.output)?;
            let transaction_id = TransactionId::from_str(&output.transaction_id)?;
            // check if we know the transaction that created this output and if we created it (if we store incoming
            // transactions separated, then this check wouldn't be required)
            let remainder = {
                match account.transactions.get(&transaction_id) {
                    Some(tx) => !tx.incoming,
                    None => false,
                }
            };
            Ok(OutputData {
                output_id: OutputId::new(transaction_id, output.output_index)?,
                message_id: MessageId::from_str(&output.message_id)?,
                amount,
                is_spent: output.is_spent,
                address,
                kind: output_kind,
                network_id,
                remainder,
                // get this from the milestone that confirmed the message with the transaction?
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis(),
            })
        })
        .collect::<crate::Result<Vec<OutputData>>>()
}

/// Get the current output ids for provided addresses
pub(crate) async fn get_outputs(
    account_handle: &AccountHandle,
    options: &SyncOptions,
    output_ids: Vec<OutputId>,
) -> crate::Result<Vec<OutputResponse>> {
    log::debug!("[SYNC] start get_outputs");
    let get_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client = crate::client::get_client().await?;
    drop(account);

    let mut found_outputs = Vec::new();
    // We split the outputs into chunks so we don't get timeouts if we have thousands
    for output_ids_chunk in output_ids
        .chunks(PARALLEL_REQUESTS_AMOUNT)
        .map(|x: &[OutputId]| x.to_vec())
        .into_iter()
    {
        let mut tasks = Vec::new();
        for output_id in output_ids_chunk {
            let client = client.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let client = client;
                    let output = client.get_output(&UtxoInput::from(output_id)).await?;
                    crate::Result::Ok(output)
                })
                .await
            });
        }
        let results = futures::future::try_join_all(tasks).await?;
        for res in results {
            let output = res?;
            found_outputs.push(output);
        }
    }

    log::debug!(
        "[SYNC] finished get_outputs in {:.2?}",
        get_outputs_sync_start_time.elapsed()
    );
    Ok(found_outputs)
}

/// Get output kind, amount and address from an OutputDto
pub(crate) fn get_output_amount_and_address(output: &OutputDto) -> crate::Result<(u64, Address, OutputKind)> {
    match output {
        OutputDto::Treasury(_) => Err(crate::Error::InvalidOutputKind("Treasury".to_string())),
        OutputDto::SignatureLockedSingle(ref r) => match &r.address {
            AddressDto::Ed25519(addr) => {
                let output_address = Address::from(Ed25519Address::from_str(&addr.address)?);
                Ok((r.amount, output_address, OutputKind::SignatureLockedSingle))
            }
        },
        OutputDto::SignatureLockedDustAllowance(ref r) => match &r.address {
            AddressDto::Ed25519(addr) => {
                let output_address = Address::from(Ed25519Address::from_str(&addr.address)?);
                Ok((r.amount, output_address, OutputKind::SignatureLockedDustAllowance))
            }
        },
    }
}
