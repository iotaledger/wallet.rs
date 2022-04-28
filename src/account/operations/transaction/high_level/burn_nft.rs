// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_client::{
    bee_block::{
        address::NftAddress,
        output::{
            dto::OutputDto,
            unlock_condition::{
                dto::{AddressUnlockConditionDto, UnlockConditionDto},
                AddressUnlockCondition,
            },
            BasicOutputBuilder, NftId, NftOutput, Output, OutputId, OUTPUT_COUNT_MAX,
        },
        payload::transaction::TransactionId,
    },
    node_api::indexer::query_parameters::QueryParameter,
};

use crate::{
    account::{
        handle::AccountHandle,
        operations::transaction::TransactionResult,
        types::{address::AddressWrapper, AccountAddress},
        AddressGenerationOptions, RemainderValueStrategy, SyncOptions, TransactionOptions,
    },
    Error,
};

impl AccountHandle {
    /// Function to mint nft.
    pub async fn burn_nft(
        &self,
        nft_id: NftId,
        options: Option<TransactionOptions>,
    ) -> crate::Result<TransactionResult> {
        log::debug!("[TRANSFER] burn_nft");

        let gen_addr = async {
            let result: crate::Result<AccountAddress> = Ok(self
                .generate_addresses(
                    1,
                    Some(AddressGenerationOptions {
                        internal: true,
                        ..Default::default()
                    }),
                )
                .await?
                .first()
                .ok_or_else(|| crate::Error::BurningFailed("Couldn't generate an address".to_string()))?
                .clone());

            result
        };

        let address = match &options {
            None => gen_addr.await?.address,
            Some(strategy) => match &strategy.remainder_value_strategy {
                RemainderValueStrategy::ReuseAddress | RemainderValueStrategy::ChangeAddress => gen_addr.await?.address,
                RemainderValueStrategy::CustomAddress(account_address) => account_address.address.clone(),
            },
        };

        self.sweep_nft_address(NftAddress::new(nft_id), address).await?;

        let (output_id, nft_output) = self.find_nft_output(nft_id).await?;
        let custom_inputs = vec![output_id];
        let outputs = vec![Self::nft_to_basic_output(&nft_output)?];

        let options = match options {
            Some(mut options) => {
                options.custom_inputs.replace(custom_inputs);
                Some(options)
            }
            None => Some(TransactionOptions {
                custom_inputs: Some(custom_inputs),
                ..Default::default()
            }),
        };

        self.send(outputs, options).await
    }

    async fn find_nft_output(&self, nft_id: NftId) -> crate::Result<(OutputId, NftOutput)> {
        let account = self.read().await;

        let (output_id, output_data) = account
            .unspent_outputs()
            .iter()
            .find(|(&output_id, output_data)| match &output_data.output {
                Output::Nft(nft_output) => nft_output.nft_id().or_from_output_id(output_id) == nft_id,
                _ => false,
            })
            .ok_or(Error::NftNotFoundInUnspentOutputs)?;

        let nft_output = match &output_data.output {
            Output::Nft(nft_output) => nft_output.clone(),
            _ => unreachable!("We already checked that it's an nft output"),
        };

        Ok((*output_id, nft_output))
    }

    fn nft_to_basic_output(nft_output: &NftOutput) -> crate::Result<Output> {
        Ok(Output::Basic(
            BasicOutputBuilder::new_with_amount(nft_output.amount())?
                .with_feature_blocks(nft_output.feature_blocks().clone())
                .with_unlock_conditions(nft_output.unlock_conditions().clone())
                .with_native_tokens(nft_output.native_tokens().clone())
                .finish()?,
        ))
    }

    async fn sweep_nft_address(&self, nft_address: NftAddress, remainder_address: AddressWrapper) -> crate::Result<()> {
        let query_parameters = vec![QueryParameter::Address(nft_address.to_string())];
        let alias_output_ids = self.client.aliases_output_ids(query_parameters.clone()).await?;
        let basic_output_ids = self.client.output_ids(query_parameters.clone()).await?;
        let nft_output_ids = self.client.output_ids(query_parameters).await?;

        let alias_outputs = self.client.get_outputs(alias_output_ids).await?;
        let basic_outputs = self.client.get_outputs(basic_output_ids).await?;
        let nft_outputs = self.client.get_outputs(nft_output_ids).await?;

        let mut output_ids = Vec::new();
        let mut outputs = Vec::new();

        for mut output_response in alias_outputs
            .into_iter()
            .chain(basic_outputs.into_iter())
            .chain(nft_outputs.into_iter())
        {
            if output_response.is_spent {
                continue;
            }

            match &mut output_response.output {
                OutputDto::Basic(output_dto) => Self::replace_address_unlock_conditions(
                    &mut output_dto.unlock_conditions,
                    AddressUnlockCondition::new(remainder_address.inner),
                )?,
                OutputDto::Alias(output_dto) => Self::replace_address_unlock_conditions(
                    &mut output_dto.unlock_conditions,
                    AddressUnlockCondition::new(remainder_address.inner),
                )?,
                OutputDto::Nft(output_dto) => Self::replace_address_unlock_conditions(
                    &mut output_dto.unlock_conditions,
                    AddressUnlockCondition::new(remainder_address.inner),
                )?,
                OutputDto::Treasury(_) | OutputDto::Foundry(_) => continue,
            }

            let transaction_id = TransactionId::from_str(&output_response.transaction_id)?;
            let output_id = OutputId::new(transaction_id, output_response.output_index)?;
            let output = Output::try_from(&output_response.output)?;

            output_ids.push(output_id);
            outputs.push(output);

            if output_ids.len() == (OUTPUT_COUNT_MAX - 1) as usize {
                self.send_sweep_transaction(*nft_address.nft_id(), output_ids.drain(..), outputs.drain(..))
                    .await?;
                let sync_options = SyncOptions {
                    addresses: vec![remainder_address.to_bech32()],
                    ..Default::default()
                };
                self.sync(Some(sync_options)).await?;
            }
        }

        if !output_ids.is_empty() {
            self.send_sweep_transaction(*nft_address.nft_id(), output_ids.drain(..), outputs.drain(..))
                .await?;
            let sync_options = SyncOptions {
                addresses: vec![remainder_address.to_bech32()],
                ..Default::default()
            };
            self.sync(Some(sync_options)).await?;
        }

        Ok(())
    }

    async fn send_sweep_transaction(
        &self,
        nft_id: NftId,
        output_ids: impl IntoIterator<Item = OutputId>,
        outputs: impl IntoIterator<Item = Output>,
    ) -> crate::Result<()> {
        let mut output_ids = output_ids.into_iter().collect::<Vec<_>>();
        let mut outputs = outputs.into_iter().collect::<Vec<_>>();

        let (output_id, nft_output) = self.find_nft_output(nft_id).await?;
        output_ids.push(output_id);
        outputs.push(Output::Nft(nft_output));

        let transfer_options = Some(TransferOptions {
            custom_inputs: Some(output_ids),
            ..Default::default()
        });

        let transfer_result = self.send(outputs, transfer_options, false).await?;

        match transfer_result.message_id {
            Some(message_id) => {
                let _ = self.client.retry_until_included(&message_id, None, None).await?;
            }
            None => return Err(Error::BurningFailed("Unable to sweep outputs".to_string())),
        }

        Ok(())
    }

    fn replace_address_unlock_conditions(
        unlock_conditions: &mut [UnlockConditionDto],
        address_unlock: AddressUnlockCondition,
    ) -> crate::Result<()> {
        for condition in unlock_conditions.iter_mut() {
            if let UnlockConditionDto::Address(address_unlock_condition_dto) = condition {
                *address_unlock_condition_dto = AddressUnlockConditionDto {
                    kind: AddressUnlockCondition::KIND,
                    address: address_unlock.address().into(),
                };
                // There can't be more than one unlock condition type so it's okay to break
                break;
            }
        }

        Ok(())
    }
}
