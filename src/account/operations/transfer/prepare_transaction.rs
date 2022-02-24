// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "events")]
use crate::events::types::{PreparedTransactionEventData, TransactionIO, TransferProgressEvent, WalletEvent};
use crate::{
    account::{handle::AccountHandle, operations::transfer::TransferOptions},
    Result,
};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use iota_client::{
    api::PreparedTransactionData,
    bee_message::{
        input::{Input, UtxoInput},
        output::{unlock_condition::UnlockCondition, Output},
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
pub(crate) async fn prepare_transaction(
    account_handle: &AccountHandle,
    inputs: Vec<InputSigningData>,
    outputs: Vec<Output>,
    options: Option<TransferOptions>,
) -> crate::Result<PreparedTransactionData> {
    log::debug!("[TRANSFER] prepare_transaction");
    let prepare_transaction_start_time = Instant::now();

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
        let input = Input::Utxo(UtxoInput::from(utxo.output_id()?));
        inputs_for_essence.push(input.clone());
        inputs_for_signing.push(utxo.clone());
        #[cfg(feature = "events")]
        inputs_for_event.push(TransactionIO {
            address: utxo.bech32_address.clone(),
            amount: output.amount(),
            remainder: None,
        })
    }

    // Build transaction essence
    let mut essence_builder = RegularTransactionEssence::builder(account_handle.client.get_network_id().await?);
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

    for output in &outputs {
        let mut address = None;
        if let Output::Basic(basic_output) = output {
            for unlock_condition in basic_output.unlock_conditions().iter() {
                if let UnlockCondition::Address(address_unlock_condition) = unlock_condition {
                    address.replace(address_unlock_condition.address());
                    break;
                }
            }
        }
        outputs_for_event.push(TransactionIO {
            address: address
                .expect("todo: update transaction events to new outputs")
                .to_bech32("iota"),
            amount: output.amount(),
            remainder: None,
        })
    }
    essence_builder = essence_builder.with_outputs(outputs);

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
            WalletEvent::TransferProgress(TransferProgressEvent::PreparedTransaction(
                PreparedTransactionEventData {
                    inputs: inputs_for_event,
                    outputs: outputs_for_event,
                    data: tagged_data,
                },
            )),
        );
    }
    log::debug!(
        "[TRANSFER] finished prepare_transaction in {:.2?}",
        prepare_transaction_start_time.elapsed()
    );
    Ok(PreparedTransactionData {
        essence,
        input_signing_data_entrys: inputs_for_signing,
    })
}
