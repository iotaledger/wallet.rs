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
    Result,
};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    keys::slip10::Chain,
};
use iota_client::{
    bee_message::{
        address::Address,
        input::{Input, UtxoInput},
        output::{
            unlock_condition::{AddressUnlockCondition, UnlockCondition},
            BasicOutputBuilder, Output,
        },
        payload::{
            transaction::{RegularTransactionEssence, TransactionEssence},
            Payload,
        },
    },
    packable::PackableExt,
    signing::types::InputSigningData,
};

use std::time::Instant;

/// Function to build the transaction essence
pub(crate) async fn create_transaction(
    account_handle: &AccountHandle,
    inputs: Vec<OutputData>,
    outputs: Vec<TransferOutput>,
    options: Option<TransferOptions>,
) -> crate::Result<(TransactionEssence, Vec<InputSigningData>, Option<Remainder>)> {
    log::debug!("[TRANSFER] create_transaction");
    let create_transaction_start_time = Instant::now();

    let mut total_input_amount = 0;
    let mut inputs_for_essence: Vec<Input> = Vec::new();
    let mut inputs_for_signing: Vec<InputSigningData> = Vec::new();
    #[cfg(feature = "events")]
    let mut inputs_for_event: Vec<TransactionIO> = Vec::new();
    #[cfg(feature = "events")]
    let mut outputs_for_event: Vec<TransactionIO> = Vec::new();
    let coin_type = { account_handle.read().await.coin_type };
    let account_index = { account_handle.read().await.index };
    let addresses = account_handle.list_addresses_with_balance().await?;

    for utxo in &inputs {
        let output = Output::try_from(&utxo.output_response.output)?;
        total_input_amount += output.amount();
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

        // 44 is for BIP 44 (HD wallets) and 4218 is the registered index for IOTA https://github.com/satoshilabs/slips/blob/master/slip-0044.md
        let chain = Chain::from_u32_hardened(vec![
            44,
            coin_type,
            account_index,
            associated_address.internal as u32,
            associated_address.key_index,
        ]);

        inputs_for_signing.push(InputSigningData {
            output_response: utxo.output_response.clone(),
            chain: Some(chain),
            bech32_address: associated_address.address.to_bech32(),
        });
        #[cfg(feature = "events")]
        inputs_for_event.push(TransactionIO {
            address: associated_address.address.to_bech32(),
            amount: output.amount(),
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
            Some(crate::account::types::OutputKind::Basic) | None => {
                outputs_for_essence.push(Output::Basic(
                    BasicOutputBuilder::new(output.amount)?
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                        .finish()?,
                ));
            }
            // todo handle other outputs
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
            Some(OutputKind::Basic) | None => {
                outputs_for_essence.push(Output::Basic(
                    BasicOutputBuilder::new(remainder_value)?
                        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                            remainder_address.address.inner,
                        )))
                        .finish()?,
                ));
            }
            _ => {
                todo!("handle other outputs")
            }
        }
    }

    let client = crate::client::get_client().await?;
    // Build transaction essence
    let mut essence_builder = RegularTransactionEssence::builder(client.get_network_id().await?);
    essence_builder = essence_builder.with_inputs(inputs_for_essence);

    let input_outputs = inputs_for_signing
        .iter()
        .map(|i| Ok(Output::try_from(&i.output_response.output)?.pack_to_vec()))
        .collect::<Result<Vec<Vec<u8>>>>()?;
    let input_outputs = input_outputs.into_iter().flatten().collect::<Vec<u8>>();
    let inputs_commitment = Blake2b256::digest(&input_outputs)
        .try_into()
        .map_err(|_e| crate::Error::Blake2b256("Hashing outputs for inputs_commitment failed."))?;
    essence_builder = essence_builder.with_inputs_commitment(inputs_commitment);

    essence_builder = essence_builder.with_outputs(outputs_for_essence);

    // Optional add a tagged payload
    #[cfg(feature = "events")]
    let mut tagged_data: Option<String> = None;
    if let Some(options) = options {
        if let Some(tagged_data_payload) = &options.tagged_data_payload {
            #[cfg(feature = "events")]
            {
                tagged_data = Some(hex::encode(tagged_data_payload.data()));
            }
            essence_builder = essence_builder.with_payload(Payload::TaggedData(Box::new(tagged_data_payload.clone())));
        }
    }

    let essence = essence_builder.finish()?;
    let essence = TransactionEssence::Regular(essence);

    #[cfg(feature = "events")]
    {
        let account_index = account_handle.read().await.index;
        account_handle.event_emitter.lock().await.emit(
            account_index,
            WalletEvent::TransferProgress(TransferProgressEvent::PreparedTransaction(PreparedTransactionData {
                inputs: inputs_for_event,
                outputs: outputs_for_event,
                data: tagged_data,
            })),
        );
    }
    log::debug!(
        "[TRANSFER] finished create_transaction in {:.2?}",
        create_transaction_start_time.elapsed()
    );
    Ok((essence, inputs_for_signing, remainder))
}
