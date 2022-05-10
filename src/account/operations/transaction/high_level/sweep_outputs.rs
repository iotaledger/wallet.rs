// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        handle::AccountHandle,
        types::{address::AddressWrapper, AccountAddress},
        AddressGenerationOptions, RemainderValueStrategy, TransferOptions,
    },
    Error,
};

use iota_client::{
    bee_message::{
        address::Address,
        output::{
            dto::OutputDto,
            unlock_condition::{
                dto::{AddressUnlockConditionDto, UnlockConditionDto},
                AddressUnlockCondition,
            },
            AliasId, AliasOutput, AliasOutputBuilder, NftId, NftOutput, NftOutputBuilder, Output, OutputId,
            OUTPUT_COUNT_MAX,
        },
        payload::transaction::TransactionId,
    },
    node_api::indexer::query_parameters::QueryParameter,
};

use std::str::FromStr;

#[allow(dead_code)]
impl AccountHandle {
    pub(crate) async fn get_sweep_remainder_address(
        &self,
        options: &Option<TransferOptions>,
    ) -> crate::Result<AddressWrapper> {
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

        let address = match options {
            None => gen_addr.await?.address,
            Some(strategy) => match &strategy.remainder_value_strategy {
                RemainderValueStrategy::ReuseAddress | RemainderValueStrategy::ChangeAddress => gen_addr.await?.address,
                RemainderValueStrategy::CustomAddress(account_address) => account_address.address.clone(),
            },
        };

        Ok(address)
    }

    pub(crate) async fn find_nft_output(&self, nft_id: NftId) -> crate::Result<(OutputId, NftOutput)> {
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

    pub(crate) async fn find_alias_output(&self, alias_id: AliasId) -> crate::Result<(OutputId, AliasOutput)> {
        let account = self.read().await;

        let (output_id, output_data) = account
            .unspent_outputs()
            .iter()
            .find(|(&output_id, output_data)| match &output_data.output {
                Output::Alias(alias_output) => alias_output.alias_id().or_from_output_id(output_id) == alias_id,
                _ => false,
            })
            .ok_or_else(|| Error::BurningFailed("Alias output not found".to_string()))?;

        let alias_output = match &output_data.output {
            Output::Alias(alias_output) => alias_output.clone(),
            _ => unreachable!("We already checked that it's an alias output"),
        };

        Ok((*output_id, alias_output))
    }

    pub(crate) async fn sweep_address_outputs(
        &self,
        address: Address,
        remainder_address: AddressWrapper,
    ) -> crate::Result<()> {
        let alias_query_parameters = vec![
            QueryParameter::Governor(address.to_bech32(remainder_address.bech32_hrp())),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ];

        let query_parameters = vec![
            QueryParameter::Address(address.to_bech32(remainder_address.bech32_hrp())),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ];

        let alias_output_ids = self.client.aliases_output_ids(alias_query_parameters).await?;
        let basic_output_ids = self.client.output_ids(query_parameters.clone()).await?;
        let nft_output_ids = self.client.nfts_output_ids(query_parameters).await?;

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
                OutputDto::Basic(output_dto) => {
                    Self::replace_unlock_conditions(&mut output_dto.unlock_conditions, &remainder_address.inner);
                }
                OutputDto::Alias(output_dto) => {
                    Self::replace_unlock_conditions(&mut output_dto.unlock_conditions, &remainder_address.inner);
                }
                OutputDto::Nft(output_dto) => {
                    Self::replace_unlock_conditions(&mut output_dto.unlock_conditions, &remainder_address.inner);
                }
                OutputDto::Treasury(_) | OutputDto::Foundry(_) => continue,
            }

            let transaction_id = TransactionId::from_str(&output_response.transaction_id)?;
            let output_id = OutputId::new(transaction_id, output_response.output_index)?;
            let output = Output::try_from(&output_response.output)?;

            output_ids.push(output_id);
            outputs.push(output);

            if output_ids.len() == (OUTPUT_COUNT_MAX - 1) as usize {
                self.send_sweep_transaction(address, output_ids.drain(..), outputs.drain(..))
                    .await?;
            }
        }

        if !output_ids.is_empty() {
            self.send_sweep_transaction(address, output_ids.drain(..), outputs.drain(..))
                .await?;
        }

        Ok(())
    }

    pub(crate) async fn send_sweep_transaction(
        &self,
        address: Address,
        output_ids: impl IntoIterator<Item = OutputId>,
        outputs: impl IntoIterator<Item = Output>,
    ) -> crate::Result<()> {
        let mut custom_inputs = Vec::new();
        let mut custom_outputs = Vec::new();

        match address {
            Address::Alias(alias_address) => {
                let (output_id, alias_output) = self.find_alias_output(*alias_address.alias_id()).await?;
                let alias_output =
                    AliasOutputBuilder::new_with_amount(alias_output.amount(), *alias_output.alias_id())?
                        .with_native_tokens(alias_output.native_tokens().clone())
                        .with_state_index(alias_output.state_index() + 1)
                        .with_state_metadata(alias_output.state_metadata().to_vec())
                        .with_foundry_counter(alias_output.foundry_counter())
                        .with_unlock_conditions(alias_output.unlock_conditions().clone())
                        .with_feature_blocks(alias_output.feature_blocks().clone())
                        .with_immutable_feature_blocks(alias_output.immutable_feature_blocks().clone())
                        .finish()?;
                custom_inputs.push(output_id);
                custom_outputs.push(Output::Alias(alias_output));
            }
            Address::Nft(nft_address) => {
                let (output_id, nft_output) = self.find_nft_output(*nft_address.nft_id()).await?;
                let nft_output = NftOutputBuilder::new_with_amount(nft_output.amount(), *nft_output.nft_id())?
                    .with_native_tokens(nft_output.native_tokens().clone())
                    .with_unlock_conditions(nft_output.unlock_conditions().clone())
                    .with_feature_blocks(nft_output.feature_blocks().clone())
                    .with_immutable_feature_blocks(nft_output.immutable_feature_blocks().clone())
                    .finish()?;
                custom_inputs.push(output_id);
                custom_outputs.push(Output::Nft(nft_output));
            }
            Address::Ed25519(_) => {
                return Err(Error::BurningFailed(
                    "Ed25519 address is not intended to be swept".to_string(),
                ));
            }
        }

        custom_inputs.append(&mut output_ids.into_iter().collect::<Vec<_>>());
        custom_outputs.append(&mut outputs.into_iter().collect::<Vec<_>>());

        let transfer_options = Some(TransferOptions {
            custom_inputs: Some(custom_inputs),
            ..Default::default()
        });

        let transfer_result = self.send(custom_outputs, transfer_options, false).await?;
        match &transfer_result.message_id {
            Some(message_id) => {
                let _ = self.client.retry_until_included(message_id, None, None).await?;
                let _ = self.sync(None).await?;
            }
            None => return Err(Error::BurningFailed("Could not sweep address outputs".to_string())),
        }

        Ok(())
    }

    fn replace_unlock_conditions(unlock_conditions: &mut Vec<UnlockConditionDto>, address: &Address) {
        unlock_conditions.clear();
        unlock_conditions.push(UnlockConditionDto::Address(AddressUnlockConditionDto {
            kind: AddressUnlockCondition::KIND,
            address: address.into(),
        }));
    }
}
