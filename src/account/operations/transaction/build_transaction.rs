// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use iota_client::{
    api::{
        input_selection::types::SelectedTransactionData, transaction::validate_regular_transaction_essence_length,
        PreparedTransactionData,
    },
    block::{
        input::{Input, UtxoInput},
        output::{unlock_condition::UnlockCondition, InputsCommitment, Output},
        payload::{
            transaction::{RegularTransactionEssence, TransactionEssence},
            Payload,
        },
    },
    secret::types::InputSigningData,
};

use crate::account::{handle::AccountHandle, operations::transaction::TransactionOptions};

impl AccountHandle {
    /// Function to build the transaction essence from the selected in and outputs
    pub(crate) fn build_transaction_essence(
        &self,
        selected_transaction_data: SelectedTransactionData,
        options: Option<TransactionOptions>,
    ) -> crate::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] build_transaction");

        let build_transaction_essence_start_time = Instant::now();
        let protocol_parameters = self.client.get_protocol_parameters()?;

        let mut inputs_for_essence: Vec<Input> = Vec::new();
        let mut inputs_for_signing: Vec<InputSigningData> = Vec::new();

        for utxo in &selected_transaction_data.inputs {
            let input = Input::Utxo(UtxoInput::from(utxo.output_id()?));
            inputs_for_essence.push(input.clone());
            inputs_for_signing.push(utxo.clone());
        }

        // Build transaction essence

        let input_outputs = inputs_for_signing
            .iter()
            .map(|i| i.output.clone())
            .collect::<Vec<Output>>();
        let inputs_commitment = InputsCommitment::new(input_outputs.iter());
        let mut essence_builder =
            RegularTransactionEssence::builder(protocol_parameters.network_id(), inputs_commitment);
        essence_builder = essence_builder.with_inputs(inputs_for_essence);

        for output in &selected_transaction_data.outputs {
            let mut address = None;
            if let Output::Basic(basic_output) = output {
                for unlock_condition in basic_output.unlock_conditions().iter() {
                    if let UnlockCondition::Address(address_unlock_condition) = unlock_condition {
                        address.replace(address_unlock_condition.address());
                        break;
                    }
                }
            }
        }
        essence_builder = essence_builder.with_outputs(selected_transaction_data.outputs);

        // Optional add a tagged payload
        if let Some(options) = options {
            if let Some(tagged_data_payload) = &options.tagged_data_payload {
                essence_builder =
                    essence_builder.with_payload(Payload::TaggedData(Box::new(tagged_data_payload.clone())));
            }
        }

        let essence = essence_builder.finish(&protocol_parameters)?;

        validate_regular_transaction_essence_length(&essence)?;

        let essence = TransactionEssence::Regular(essence);

        let prepared_transaction_data = PreparedTransactionData {
            essence,
            inputs_data: inputs_for_signing,
            remainder: selected_transaction_data.remainder,
        };

        log::debug!(
            "[TRANSACTION] finished build_transaction in {:.2?}",
            build_transaction_essence_start_time.elapsed()
        );
        Ok(prepared_transaction_data)
    }
}
