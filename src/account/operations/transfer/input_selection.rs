// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    constants::MIN_DUST_ALLOWANCE_VALUE,
    handle::AccountHandle,
    types::{OutputData, OutputKind},
};
#[cfg(feature = "events")]
use crate::events::types::{TransferProgressEvent, WalletEvent};

use iota_client::bee_message::{
    constants::{INPUT_OUTPUT_COUNT_MAX, INPUT_OUTPUT_COUNT_RANGE},
    output::OutputId,
};

/// Selects inputs for a transaction and locks them in the account, so they don't get used again
pub(crate) async fn select_inputs(
    account_handle: &AccountHandle,
    amount_to_send: u64,
    custom_inputs: Option<Vec<OutputId>>,
) -> crate::Result<Vec<OutputData>> {
    log::debug!("[TRANSFER] select_inputs");
    let mut account = account_handle.write().await;
    #[cfg(feature = "events")]
    account_handle.event_emitter.lock().await.emit(
        account.index,
        WalletEvent::TransferProgress(TransferProgressEvent::SelectingInputs),
    );

    // if custom inputs are provided we should only use them (validate if we have the outputs in this account and
    // that the amount is enough)
    if let Some(custom_inputs) = custom_inputs {
        let mut total_input_amount = 0;
        let mut inputs = Vec::new();
        for input in custom_inputs {
            if account.locked_outputs.contains(&input) {
                return Err(crate::Error::CustomInputError(format!(
                    "{} already used in another transaction",
                    input
                )));
            }
            match account.unspent_outputs.get(&input) {
                Some(output) => {
                    total_input_amount += output.amount;
                    inputs.push(output.clone());
                }
                None => return Err(crate::Error::CustomInputError(format!("Unknown input: {}", input))),
            }
        }
        if total_input_amount != amount_to_send {
            return Err(crate::Error::CustomInputError(format!(
                "Inputs amount {} doesn't match amount to send {}",
                total_input_amount, amount_to_send,
            )));
        }
        // lock outputs so they don't get used by another transaction
        for output in &inputs {
            account.locked_outputs.insert(output.output_id);
        }
        return Ok(inputs);
    }

    let client = crate::client::get_client().await?;
    let network_id = client.get_network_id().await?;

    let mut signature_locked_outputs = Vec::new();
    let mut dust_allowance_outputs = Vec::new();
    for (output_id, output) in account.unspent_outputs.iter() {
        // check if not in pending transaction (locked_outputs) and if from the correct network
        if !output.is_spent && !account.locked_outputs.contains(output_id) && output.network_id == network_id {
            match output.kind {
                OutputKind::SignatureLockedSingle => signature_locked_outputs.push(output),
                OutputKind::SignatureLockedDustAllowance => dust_allowance_outputs.push(output),
                _ => {}
            }
        }
    }

    // todo try to select matching inputs first, only if that's not possible we should select the inputs like below

    // Sort inputs so we can get the biggest inputs first and don't reach the input limit, if we don't have the
    // funds spread over too many outputs
    signature_locked_outputs.sort_by(|a, b| b.amount.cmp(&a.amount));
    dust_allowance_outputs.sort_by(|a, b| b.amount.cmp(&a.amount));

    let mut input_sum = 0;
    let selected_outputs: Vec<OutputData> = signature_locked_outputs
        .into_iter()
        // add dust_allowance_outputs only at the end so we don't try to move them when we might have still dust
        .chain(dust_allowance_outputs.into_iter())
        .take_while(|input| {
            let value = input.amount;
            let old_sum = input_sum;
            input_sum += value;
            old_sum < amount_to_send
                || (old_sum - amount_to_send < MIN_DUST_ALLOWANCE_VALUE && old_sum != amount_to_send)
        })
        .cloned()
        .collect();

    // recalculate the input sum, because during take_while() we maybe also added a last output, even if it didn't get
    // added anymore
    let selected_input_sum: u64 = selected_outputs.iter().map(|o| o.amount).sum();
    if selected_input_sum < amount_to_send {
        return Err(crate::Error::InsufficientFunds(selected_input_sum, amount_to_send));
    }
    let remainder_value = selected_input_sum - amount_to_send;
    if remainder_value != 0 && remainder_value < MIN_DUST_ALLOWANCE_VALUE {
        return Err(crate::Error::LeavingDustError(format!(
            "Transaction would leave dust behind ({}i)",
            remainder_value
        )));
    }
    if !INPUT_OUTPUT_COUNT_RANGE.contains(&selected_outputs.len()) {
        #[cfg(feature = "events")]
        account_handle
            .event_emitter
            .lock()
            .await
            .emit(account.index, WalletEvent::ConsolidationRequired);
        return Err(crate::Error::ConsolidationRequired(
            selected_outputs.len(),
            INPUT_OUTPUT_COUNT_MAX,
        ));
    }

    // lock outputs so they don't get used by another transaction
    for output in &selected_outputs {
        // log::debug!(
        //     "[TRANSFER] select_inputs: lock {}",
        //     output.output_id,
        // );
        account.locked_outputs.insert(output.output_id);
    }
    Ok(selected_outputs)
}
