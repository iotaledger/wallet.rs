// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Borrow, collections::HashSet, pin::Pin, str::FromStr};

use futures::{Future, FutureExt};
use iota_client::{
    api::ClientMessageBuilder,
    bee_message::{
        address::{Address, AliasAddress, NftAddress},
        output::{
            dto::OutputDto,
            unlock_condition::{
                dto::{AddressUnlockConditionDto, UnlockConditionDto},
                AddressUnlockCondition,
            },
            AliasId, AliasOutput, AliasOutputBuilder, FoundryId, NftId, NftOutput, Output, OutputId, TokenScheme,
            OUTPUT_COUNT_MAX,
        },
        payload::transaction::TransactionId,
    },
    bee_rest_api::types::responses::OutputResponse,
    node_api::indexer::query_parameters::QueryParameter,
};

use crate::{
    account::{
        handle::AccountHandle,
        types::{address::AddressWrapper, AccountAddress, OutputData},
        AddressGenerationOptions, RemainderValueStrategy, TransferOptions,
    },
    Error,
};

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

    async fn output_id_and_nft_output(&self, nft_id: NftId) -> crate::Result<(OutputId, NftOutput)> {
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

    async fn output_id_and_next_alias_output_state(&self, alias_id: AliasId) -> crate::Result<(OutputId, AliasOutput)> {
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
            Output::Alias(alias_output) => alias_output,
            _ => unreachable!("We already checked that it's an alias output"),
        };

        let new_state_alias_output = AliasOutputBuilder::from(alias_output)
            .with_alias_id(alias_id)
            .with_state_index(alias_output.state_index() + 1)
            .finish()?;

        Ok((*output_id, new_state_alias_output))
    }

    pub(crate) fn sweep_address_outputs<'a>(
        &'a self,
        address: Address,
        remainder_address: &'a AddressWrapper,
    ) -> Pin<Box<dyn Future<Output = crate::Result<Vec<TransactionId>>> + Send + 'a>> {
        async move {
            let address = AddressWrapper::new(address, remainder_address.bech32_hrp().to_string());

            let alias_outputs = self.fetch_governor_address_alias_outputs(&address).await?;
            let basic_outputs = self.fetch_address_basic_outputs(&address).await?;
            let nft_outputs = self.fetch_address_nft_outputs(&address).await?;

            let mut output_ids = Vec::new();
            let mut outputs = Vec::new();
            let mut transaction_ids = Vec::new();

            let network_id = self.client.get_network_id().await?;

            for mut output_response in alias_outputs
                .into_iter()
                .chain(basic_outputs.into_iter())
                .chain(nft_outputs.into_iter())
            {
                if output_response.is_spent {
                    continue;
                }

                let transaction_id = TransactionId::from_str(&output_response.transaction_id)?;
                let output_id = OutputId::new(transaction_id, output_response.output_index)?;
                self.update_unspent_output(output_response.clone(), output_id, network_id)
                    .await?;

                match &mut output_response.output {
                    OutputDto::Basic(output_dto) => {
                        replace_unlock_conditions(&mut output_dto.unlock_conditions, &remainder_address.inner);
                    }
                    OutputDto::Alias(alias_dto) => {
                        let mut alias_id: AliasId = alias_dto.alias_id.borrow().try_into()?;
                        if alias_id.is_null() {
                            alias_id = AliasId::from(output_id);
                        }
                        // Recursively sweep alias address outputs
                        let txn_ids = self
                            .sweep_address_outputs(Address::Alias(AliasAddress::new(alias_id)), remainder_address)
                            .await?;
                        transaction_ids.extend(txn_ids);
                        replace_unlock_conditions(&mut alias_dto.unlock_conditions, &remainder_address.inner);
                    }
                    OutputDto::Nft(nft_dto) => {
                        let mut nft_id: NftId = nft_dto.nft_id.borrow().try_into()?;
                        if nft_id.is_null() {
                            nft_id = NftId::from(output_id)
                        }
                        // Recursively sweep nft address outputs
                        let txn_ids = self
                            .sweep_address_outputs(Address::Nft(NftAddress::new(nft_id)), remainder_address)
                            .await?;
                        transaction_ids.extend(txn_ids);
                        replace_unlock_conditions(&mut nft_dto.unlock_conditions, &remainder_address.inner);
                    }
                    // Didn't ask for treasury and foundry outputs
                    OutputDto::Treasury(_) | OutputDto::Foundry(_) => continue,
                }

                let output = Output::try_from(&output_response.output)?;

                output_ids.push(output_id);
                outputs.push(output);

                if output_ids.len() == (OUTPUT_COUNT_MAX - 1) as usize {
                    let transaction_id = self
                        .send_sweep_transaction(address.clone(), output_ids.drain(..), outputs.drain(..))
                        .await?;
                    transaction_ids.push(transaction_id);
                }
            }

            if !output_ids.is_empty() {
                let transaction_id = self
                    .send_sweep_transaction(address.clone(), output_ids.drain(..), outputs.drain(..))
                    .await?;
                transaction_ids.push(transaction_id);
            }

            //  Fetch and burn all foundries we can find
            if let Address::Alias(alias_address) = &address.inner {
                let _ = self
                    .destroy_foundries(remainder_address.bech32_hrp(), alias_address)
                    .await?;
            }

            Ok(transaction_ids)
        }
        .boxed()
    }

    pub(crate) async fn send_sweep_transaction(
        &self,
        address: AddressWrapper,
        output_ids: impl IntoIterator<Item = OutputId>,
        outputs: impl IntoIterator<Item = Output>,
    ) -> crate::Result<TransactionId> {
        let mut custom_inputs = Vec::new();
        let mut custom_outputs = Vec::new();

        match address.inner {
            Address::Alias(alias_address) => {
                let (output_id, alias_output) = self
                    .output_id_and_next_alias_output_state(*alias_address.alias_id())
                    .await?;
                custom_inputs.push(output_id);
                custom_outputs.push(Output::Alias(alias_output));
            }
            Address::Nft(nft_address) => {
                let (output_id, nft_output) = self.output_id_and_nft_output(*nft_address.nft_id()).await?;
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
            try_fetch_unfound_outputs: true,
            ..Default::default()
        });

        let transfer_result = self.send(custom_outputs, transfer_options).await?;
        match &transfer_result.message_id {
            Some(message_id) => {
                let _ = self.client.retry_until_included(message_id, None, None).await?;
                let _ = self.sync(None).await?;
            }
            None => return Err(Error::BurningFailed("Could not sweep address outputs".to_string())),
        }

        Ok(transfer_result.transaction_id)
    }

    async fn fetch_governor_address_alias_outputs(
        &self,
        address: &AddressWrapper,
    ) -> crate::Result<Vec<OutputResponse>> {
        let alias_query_parameters = vec![QueryParameter::Governor(address.to_bech32())];

        let alias_output_ids = self.client.aliases_output_ids(alias_query_parameters).await?;
        let output_responses = self.client.get_outputs(alias_output_ids).await?;

        Ok(output_responses)
    }

    async fn fetch_address_basic_outputs(&self, address: &AddressWrapper) -> crate::Result<Vec<OutputResponse>> {
        let query_parameters = vec![
            QueryParameter::Address(address.to_bech32()),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ];

        let basic_output_ids = self.client.output_ids(query_parameters.clone()).await?;
        let output_responses = self.client.get_outputs(basic_output_ids).await?;

        Ok(output_responses)
    }

    async fn fetch_address_nft_outputs(&self, address: &AddressWrapper) -> crate::Result<Vec<OutputResponse>> {
        let query_parameters = vec![
            QueryParameter::Address(address.to_bech32()),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ];

        let nfts_output_ids = self.client.nfts_output_ids(query_parameters).await?;
        let output_responses = self.client.get_outputs(nfts_output_ids).await?;

        Ok(output_responses)
    }

    async fn update_unspent_output(
        &self,
        output_response: OutputResponse,
        output_id: OutputId,
        network_id: u64,
    ) -> crate::Result<()> {
        let mut account = self.write().await;

        let (amount, address) = ClientMessageBuilder::get_output_amount_and_address(&output_response.output, None)?;
        // check if we know the transaction that created this output and if we created it (if we store incoming
        // transactions separated, then this check wouldn't be required)
        let remainder = {
            match account.transactions.get(output_id.transaction_id()) {
                Some(tx) => !tx.incoming,
                None => false,
            }
        };

        let output = Output::try_from(&output_response.output)?;

        let output_data = OutputData {
            output_id,
            output,
            is_spent: output_response.is_spent,
            output_response,
            amount,
            address,
            network_id,
            remainder,
            chain: None,
        };

        account.unspent_outputs.entry(output_id).or_insert(output_data);

        Ok(())
    }

    async fn update_unspent_outputs(&self, output_responses: Vec<OutputResponse>) -> crate::Result<()> {
        let network_id = self.client.get_network_id().await?;
        let mut account = self.write().await;

        for output_response in output_responses.into_iter() {
            let transaction_id = TransactionId::from_str(&output_response.transaction_id)?;
            let output_id = OutputId::new(transaction_id, output_response.output_index)?;
            let (amount, address) = ClientMessageBuilder::get_output_amount_and_address(&output_response.output, None)?;
            // check if we know the transaction that created this output and if we created it (if we store incoming
            // transactions separated, then this check wouldn't be required)
            let remainder = {
                match account.transactions.get(&transaction_id) {
                    Some(tx) => !tx.incoming,
                    None => false,
                }
            };

            let output_data = OutputData {
                output_id,
                output: Output::try_from(&output_response.output)?,
                is_spent: output_response.is_spent,
                output_response,
                amount,
                address,
                network_id,
                remainder,
                chain: None,
            };
            account.unspent_outputs.entry(output_id).or_insert(output_data);
        }

        Ok(())
    }

    async fn destroy_foundries(&self, hrp: &str, alias_address: &AliasAddress) -> crate::Result<Vec<TransactionId>> {
        let foundries_query_parameters = vec![
            QueryParameter::AliasAddress(Address::Alias(*alias_address).to_bech32(hrp)),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ];

        let foundry_output_ids = self.client.foundries_output_ids(foundries_query_parameters).await?;
        let output_responses = self.client.get_outputs(foundry_output_ids).await?;
        let mut foundry_ids = HashSet::new();

        for output_response in &output_responses {
            match &output_response.output {
                OutputDto::Foundry(foundry_output) => {
                    let token_scheme: TokenScheme = foundry_output.token_scheme.borrow().try_into()?;
                    let foundry_id = FoundryId::build(alias_address, foundry_output.serial_number, token_scheme.kind());
                    foundry_ids.insert(foundry_id);
                }
                _ => return Err(Error::BurningFailed("Unexpected non-foundry output".to_string())),
            }
        }

        self.update_unspent_outputs(output_responses).await?;

        let transfer_options = Some(TransferOptions {
            allow_burning: true,
            try_fetch_unfound_outputs: true,
            ..Default::default()
        });

        self.burn_foundries(foundry_ids, transfer_options).await
    }
}

fn replace_unlock_conditions(unlock_conditions: &mut Vec<UnlockConditionDto>, address: &Address) {
    unlock_conditions.clear();
    unlock_conditions.push(UnlockConditionDto::Address(AddressUnlockConditionDto {
        kind: AddressUnlockCondition::KIND,
        address: address.into(),
    }));
}
