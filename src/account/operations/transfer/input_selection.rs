// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::handle::AccountHandle;
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::{
    api::input_selection::{try_select_inputs, types::SelectedTransactionData},
    bee_message::{address::Address, input::INPUT_COUNT_MAX, output::Output},
    signing::types::InputSigningData,
};

/// Selects inputs for a transaction and locks them in the account, so they don't get used again
pub(crate) async fn select_inputs(
    account_handle: &AccountHandle,
    outputs: Vec<Output>,
    custom_inputs: Option<Vec<InputSigningData>>,
    remainder_address: Option<Address>,
) -> crate::Result<SelectedTransactionData> {
    log::debug!("[TRANSFER] select_inputs");
    // lock so the same inputs can't be selected in multiple transfers
    let mut account = account_handle.write().await;
    #[cfg(feature = "events")]
    account_handle.event_emitter.lock().await.emit(
        account.index,
        WalletEvent::TransferProgress(TransferProgressEvent::SelectingInputs),
    );

    // if custom inputs are provided we should only use them (validate if we have the outputs in this account and
    // that the amount is enough)
    if let Some(custom_inputs) = custom_inputs {
        let selected_transaction_data = try_select_inputs(custom_inputs, outputs, true, remainder_address).await?;

        // lock outputs so they don't get used by another transaction
        for output in &selected_transaction_data.inputs {
            account.locked_outputs.insert(output.output_id()?);
        }
        return Ok(selected_transaction_data);
    }

    let network_id = account_handle.client.get_network_id().await?;

    let mut available_outputs = Vec::new();
    for (output_id, output) in account.unspent_outputs.iter() {
        // check if not in pending transaction (locked_outputs) and if from the correct network
        if !output.is_spent && !account.locked_outputs.contains(output_id) && output.network_id == network_id {
            available_outputs.push(output.input_signing_data()?);
        }
    }

    let selected_transaction_data = match try_select_inputs(available_outputs, outputs, false, remainder_address).await
    {
        Ok(r) => r,
        Err(iota_client::Error::ConsolidationRequired(output_count)) => {
            #[cfg(feature = "events")]
            account_handle
                .event_emitter
                .lock()
                .await
                .emit(account.index, WalletEvent::ConsolidationRequired);
            return Err(crate::Error::ConsolidationRequired(output_count, INPUT_COUNT_MAX));
        }
        Err(e) => return Err(e.into()),
    };

    // lock outputs so they don't get used by another transaction
    for output in &selected_transaction_data.inputs {
        log::debug!("[TRANSFER] locking: {}", output.output_id()?);
        account.locked_outputs.insert(output.output_id()?);
    }
    Ok(selected_transaction_data)
}
