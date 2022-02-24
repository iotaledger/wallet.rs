// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// transfer or transaction?

mod input_selection;
mod options;
mod prepare_transaction;
mod sign_transaction;
pub(crate) mod submit_transaction;

use crate::{
    account::{
        handle::AccountHandle,
        types::{address::AccountAddress, InclusionState, Transaction},
        AddressGenerationOptions,
    },
    events::types::{AddressData, TransferProgressEvent, WalletEvent},
};
use input_selection::select_inputs;

use iota_client::{
    bee_message::{
        input::INPUT_COUNT_RANGE,
        output::{Output, OUTPUT_COUNT_RANGE},
        payload::transaction::{TransactionId, TransactionPayload},
        MessageId,
    },
    signing::types::InputSigningData,
};

pub use options::{RemainderValueStrategy, TransferOptions};
use packable::bounded::TryIntoBoundedU16Error;
use serde::Serialize;

use std::time::{SystemTime, UNIX_EPOCH};

/// The result of a transfer, message_id is an option because submitting the transaction could fail
#[derive(Debug, Serialize)]
pub struct TransferResult {
    pub transaction_id: TransactionId,
    pub message_id: Option<MessageId>,
}

// Data for signing metadata (used for ledger signer)
pub(crate) struct Remainder {
    address: AccountAddress,
    amount: u64,
}

/// Function to create a transfer to provided outputs, the options can define the RemainderValueStrategy or custom
/// inputs.
pub async fn send_transfer(
    account_handle: &AccountHandle,
    outputs: Vec<Output>,
    options: Option<TransferOptions>,
) -> crate::Result<TransferResult> {
    log::debug!("[TRANSFER] send_transfer");
    // validate amounts
    if !OUTPUT_COUNT_RANGE.contains(&(outputs.len() as u16)) {
        return Err(crate::Error::BeeMessage(
            iota_client::bee_message::Error::InvalidOutputCount(TryIntoBoundedU16Error::Truncated(outputs.len())),
        ));
    }

    let custom_inputs: Option<Vec<InputSigningData>> = {
        if let Some(options) = options.clone() {
            // validate inputs amount
            if let Some(inputs) = &options.custom_inputs {
                if !INPUT_COUNT_RANGE.contains(&(inputs.len() as u16)) {
                    return Err(crate::Error::BeeMessage(
                        iota_client::bee_message::Error::InvalidInputCount(TryIntoBoundedU16Error::Truncated(
                            inputs.len(),
                        )),
                    ));
                }
                let account = account_handle.read().await;
                let mut input_outputs = Vec::new();
                for output_id in inputs {
                    match account.unspent_outputs().get(output_id) {
                        Some(output) => input_outputs.push(output.input_signing_data()?),
                        None => {
                            return Err(crate::Error::CustomInputError(format!(
                                "Custom input {} not found in unspent outputs",
                                output_id
                            )))
                        }
                    }
                }
                Some(input_outputs)
            } else {
                None
            }
        } else {
            None
        }
    };

    let remainder_address = match &options {
        Some(options) => {
            match &options.remainder_value_strategy {
                RemainderValueStrategy::ReuseAddress => {
                    // select_inputs will select an address from the inputs if it's none
                    None
                }
                RemainderValueStrategy::ChangeAddress => {
                    let remainder_address = account_handle
                        .generate_addresses(
                            1,
                            Some(AddressGenerationOptions {
                                internal: true,
                                ..Default::default()
                            }),
                        )
                        .await?
                        .first()
                        .expect("Didn't generate an address")
                        .clone();
                    #[cfg(feature = "events")]
                    {
                        let account_index = account_handle.read().await.index;
                        account_handle.event_emitter.lock().await.emit(
                            account_index,
                            WalletEvent::TransferProgress(TransferProgressEvent::GeneratingRemainderDepositAddress(
                                AddressData {
                                    address: remainder_address.address.to_bech32(),
                                },
                            )),
                        );
                    }
                    Some(remainder_address.address().inner)
                }
                RemainderValueStrategy::CustomAddress(address) => Some(address.address().inner),
            }
        }
        None => None,
    };

    let selected_transaction_data = select_inputs(account_handle, outputs, custom_inputs, remainder_address).await?;
    // can we unlock the outputs in a better way if the transaction creation fails?
    let prepared_transaction_data = match prepare_transaction::prepare_transaction(
        account_handle,
        selected_transaction_data.inputs.clone(),
        selected_transaction_data.outputs.clone(),
        options,
    )
    .await
    {
        Ok(res) => res,
        Err(err) => {
            // unlock outputs so they are available for a new transaction
            unlock_inputs(account_handle, selected_transaction_data.inputs).await?;
            return Err(err);
        }
    };
    let transaction_payload = match sign_transaction::sign_tx_essence(
        account_handle,
        prepared_transaction_data.essence,
        prepared_transaction_data.input_signing_data_entrys,
        selected_transaction_data.remainder_output,
    )
    .await
    {
        Ok(res) => res,
        Err(err) => {
            // unlock outputs so they are available for a new transaction
            unlock_inputs(account_handle, selected_transaction_data.inputs).await?;
            return Err(err);
        }
    };

    let message_id =
        match submit_transaction::submit_transaction_payload(account_handle, transaction_payload.clone()).await {
            Ok(message_id) => Some(message_id),
            Err(_) => None,
        };

    // store transaction payload to account (with db feature also store the account to the db) here before sending

    let network_id = account_handle.client.get_network_id().await?;
    let transaction_id = transaction_payload.id();
    let mut account = account_handle.write().await;
    account.transactions.insert(
        transaction_id,
        Transaction {
            payload: transaction_payload,
            message_id,
            network_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis(),
            inclusion_state: InclusionState::Pending,
            incoming: false,
        },
    );
    account.pending_transactions.insert(transaction_id);
    #[cfg(feature = "storage")]
    log::debug!("[TRANSFER] storing account {}", account.index());
    crate::storage::manager::get()
        .await?
        .lock()
        .await
        .save_account(&account)
        .await?;
    Ok(TransferResult {
        transaction_id,
        message_id,
    })
}

// unlock outputs
async fn unlock_inputs(account_handle: &AccountHandle, inputs: Vec<InputSigningData>) -> crate::Result<()> {
    let mut account = account_handle.write().await;
    for input_signing_data in &inputs {
        account.locked_outputs.remove(&input_signing_data.output_id()?);
    }
    Ok(())
}
