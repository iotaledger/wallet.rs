// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{handle::AccountHandle, types::OutputData};

use iota_client::{
    api::ClientMessageBuilder,
    bee_message::{output::OutputId, payload::transaction::TransactionId, MessageId},
    bee_rest_api::types::responses::OutputResponse,
};

use std::{str::FromStr, time::Instant};

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
            let (amount, address) = ClientMessageBuilder::get_output_amount_and_address(&output.output, None)?;
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
                output_response: output.clone(),
                message_id: MessageId::from_str(&output.message_id)?,
                amount,
                is_spent: output.is_spent,
                address,
                network_id,
                remainder,
            })
        })
        .collect::<crate::Result<Vec<OutputData>>>()
}

/// Get the current output ids for provided addresses
pub(crate) async fn get_outputs(
    account_handle: &AccountHandle,
    output_ids: Vec<OutputId>,
) -> crate::Result<Vec<OutputResponse>> {
    log::debug!("[SYNC] start get_outputs");
    let get_outputs_sync_start_time = Instant::now();
    let account = account_handle.read().await;

    let client = crate::client::get_client().await?;
    drop(account);

    let found_outputs = client.get_outputs(output_ids).await?;

    log::debug!(
        "[SYNC] finished get_outputs in {:.2?}",
        get_outputs_sync_start_time.elapsed()
    );
    Ok(found_outputs)
}
