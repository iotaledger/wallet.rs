// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "events")]
use crate::events::types::{AddressData, PreparedTransactionData, TransactionIO, TransferProgressEvent, WalletEvent};
use crate::{
    account::{
        constants::MIN_DUST_ALLOWANCE_VALUE,
        handle::AccountHandle,
        operations::{
            address_generation::AddressGenerationOptions,
            transfer::{Remainder, RemainderValueStrategy, TransferOptions, TransferOutput},
        },
        types::{
            address::{AccountAddress, AddressWithBalance},
            OutputData, OutputKind,
        },
    },
    signing::TransactionInput,
};

use iota_client::{
    bee_message::{
        address::Address,
        input::{Input, UtxoInput},
        output::{Output, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput},
        payload::{
            transaction::{Essence, RegularEssence},
            Payload,
        },
    },
    common::packable::Packable,
};

use std::time::Instant;

/// Function to build the transaction essence
pub(crate) async fn create_transaction(
    account_handle: &AccountHandle,
    inputs: Vec<OutputData>,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<(Essence, Vec<TransactionInput>, Option<Remainder>)> {
    log::debug!("[TRANSFER] create_transaction");
    let create_transaction_start_time = Instant::now();

    let mut total_input_amount = 0;
    let mut inputs_for_essence: Vec<Input> = Vec::new();
    let mut inputs_for_signing: Vec<TransactionInput> = Vec::new();
    #[cfg(feature = "events")]
    let mut inputs_for_event: Vec<TransactionIO> = Vec::new();
    #[cfg(feature = "events")]
    let mut outputs_for_event: Vec<TransactionIO> = Vec::new();
    let addresses = account_handle.list_addresses_with_balance().await?;
    for utxo in &inputs {
        total_input_amount += utxo.amount;
        let input = Input::Utxo(UtxoInput::from(utxo.output_id));
        inputs_for_essence.push(input.clone());
        // instead of finding the key_index and internal by iterating over all addresses we could also add this data to
        // the OutputData struct when syncing
        let associated_address = (*addresses
            .iter()
            .filter(|a| a.address().inner == utxo.address)
            .collect::<Vec<&AddressWithBalance>>()
            .first()
            // todo: decide if we want to change the logic so we don't have to search the address or return an Error and
            // don't panic
            .expect("Didn't find input address in account"))
        .clone();
        inputs_for_signing.push(TransactionInput {
            input,
            address_index: associated_address.key_index,
            address_internal: associated_address.internal,
        });
        #[cfg(feature = "events")]
        inputs_for_event.push(TransactionIO {
            address: associated_address.address.to_bech32(),
            amount: utxo.amount,
            remainder: None,
        })
    }

    let mut total_output_amount = 0;
    let mut outputs_for_essence: Vec<Output> = Vec::new();
    for output in outputs.iter() {
        #[cfg(feature = "events")]
        outputs_for_event.push(TransactionIO {
            address: output.address.clone(),
            amount: output.amount,
            remainder: Some(false),
        });
        let address = Address::try_from_bech32(&output.address)?;
        total_output_amount += output.amount;
        match output.output_kind {
            Some(crate::account::types::OutputKind::SignatureLockedSingle) | None => {
                outputs_for_essence.push(SignatureLockedSingleOutput::new(address, output.amount)?.into());
            }
            Some(crate::account::types::OutputKind::SignatureLockedDustAllowance) => {
                outputs_for_essence.push(SignatureLockedDustAllowanceOutput::new(address, output.amount)?.into());
            }
            _ => return Err(crate::error::Error::InvalidOutputKind("Treasury".to_string())),
        }
    }

    if total_input_amount < total_output_amount {
        return Err(crate::Error::InsufficientFunds(total_input_amount, total_output_amount));
    }
    let remainder_value = total_input_amount - total_output_amount;
    if remainder_value != 0 && remainder_value < MIN_DUST_ALLOWANCE_VALUE {
        return Err(crate::Error::LeavingDustError(format!(
            "Transaction would leave dust behind ({}i)",
            remainder_value
        )));
    }

    // Add remainder output
    let mut remainder = None;
    if remainder_value != 0 {
        let options_ = options.clone().unwrap_or_default();
        let remainder_address = {
            match options_.remainder_value_strategy {
                RemainderValueStrategy::ReuseAddress => {
                    let address_with_balance = addresses
                        .iter()
                        .find(|address| address.address.inner == inputs.first().expect("no input provided").address)
                        .expect("Input address not found");
                    AccountAddress {
                        address: address_with_balance.address.clone(),
                        internal: address_with_balance.internal,
                        key_index: address_with_balance.key_index,
                        used: true,
                    }
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
                        .expect("Didn't generated an address")
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
                    remainder_address
                }
                RemainderValueStrategy::CustomAddress(address) => address,
            }
        };
        #[cfg(feature = "events")]
        outputs_for_event.push(TransactionIO {
            address: remainder_address.address.to_bech32(),
            amount: remainder_value,
            remainder: Some(true),
        });
        remainder.replace(Remainder {
            address: remainder_address.clone(),
            amount: remainder_value,
        });
        match options_.remainder_output_kind {
            Some(OutputKind::SignatureLockedDustAllowance) => outputs_for_essence.push(
                SignatureLockedDustAllowanceOutput::new(remainder_address.address.inner, remainder_value)?.into(),
            ),
            _ => outputs_for_essence
                .push(SignatureLockedSingleOutput::new(remainder_address.address.inner, remainder_value)?.into()),
        }
    }

    // Build transaction essence
    let mut essence_builder = RegularEssence::builder();

    // Order inputs and add them to the essence
    inputs_for_essence.sort_unstable_by_key(|a| a.pack_new());
    essence_builder = essence_builder.with_inputs(inputs_for_essence);

    // Order outputs and add them to the essence
    outputs_for_essence.sort_unstable_by_key(|a| a.pack_new());
    essence_builder = essence_builder.with_outputs(outputs_for_essence);

    // Optional add indexation payload
    #[cfg(feature = "events")]
    let mut indexation_data: Option<String> = None;
    if let Some(options) = options {
        if let Some(indexation) = &options.indexation {
            #[cfg(feature = "events")]
            {
                indexation_data = Some(hex::encode(indexation.data()));
            }
            essence_builder = essence_builder.with_payload(Payload::Indexation(Box::new(indexation.clone())));
        }
    }

    let essence = essence_builder.finish()?;
    let essence = Essence::Regular(essence);

    #[cfg(feature = "events")]
    {
        let account_index = account_handle.read().await.index;
        account_handle.event_emitter.lock().await.emit(
            account_index,
            WalletEvent::TransferProgress(TransferProgressEvent::PreparedTransaction(PreparedTransactionData {
                inputs: inputs_for_event,
                outputs: outputs_for_event,
                data: indexation_data,
            })),
        );
    }
    log::debug!(
        "[TRANSFER] finished create_transaction in {:.2?}",
        create_transaction_start_time.elapsed()
    );
    Ok((essence, inputs_for_signing, remainder))
}
