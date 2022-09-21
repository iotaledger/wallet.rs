// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{pin::Pin, str::FromStr};

use futures::{Future, FutureExt};
use iota_client::{
    api::ClientBlockBuilder,
    api_types::responses::OutputResponse,
    block::{
        address::Address,
        output::{
            dto::OutputDto,
            unlock_condition::{
                dto::{AddressUnlockConditionDto, UnlockConditionDto},
                AddressUnlockCondition,
            },
            AliasId, AliasOutput, AliasOutputBuilder, NftId, NftOutput, Output, OutputId,
        },
        payload::transaction::TransactionId,
    },
    node_api::indexer::query_parameters::QueryParameter,
};

use crate::{
    account::{
        handle::AccountHandle,
        types::{address::AddressWrapper, OutputData, Transaction},
        RemainderValueStrategy, TransactionOptions,
    },
    Error,
};

impl AccountHandle {
    pub(crate) async fn get_sweep_remainder_address(
        &self,
        address: &AddressWrapper,
        options: &Option<TransactionOptions>,
    ) -> crate::Result<AddressWrapper> {
        let address = match options {
            None => self.generate_remainder_address().await?.address,
            Some(strategy) => match &strategy.remainder_value_strategy {
                RemainderValueStrategy::ReuseAddress => address.clone(),
                RemainderValueStrategy::ChangeAddress => self.generate_remainder_address().await?.address,
                RemainderValueStrategy::CustomAddress(account_address) => account_address.address.clone(),
            },
        };

        Ok(address)
    }

    async fn output_id_and_nft_output(&self, nft_id: NftId) -> crate::Result<(OutputId, NftOutput)> {
        let account = self.read().await;

        let (output_id, nft_output) = account
            .unspent_outputs()
            .iter()
            .find_map(|(&output_id, output_data)| match &output_data.output {
                Output::Nft(nft_output) => {
                    if nft_output.nft_id().or_from_output_id(output_id) == nft_id {
                        Some((output_id, nft_output))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .ok_or(Error::NftNotFoundInUnspentOutputs)?;

        Ok((output_id, nft_output.clone()))
    }

    async fn output_id_and_next_alias_output_state(&self, alias_id: AliasId) -> crate::Result<(OutputId, AliasOutput)> {
        let account = self.read().await;

        let (output_id, alias_output) = account
            .unspent_outputs()
            .iter()
            .find_map(|(&output_id, output_data)| match &output_data.output {
                Output::Alias(alias_output) => {
                    if alias_output.alias_id().or_from_output_id(output_id) == alias_id {
                        Some((output_id, alias_output))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .ok_or_else(|| Error::BurningOrMeltingFailed("alias output not found".to_string()))?;

        let new_state_alias_output = AliasOutputBuilder::from(alias_output)
            .with_alias_id(alias_id)
            .with_state_index(alias_output.state_index() + 1)
            .finish()?;

        Ok((output_id, new_state_alias_output))
    }

    // Will send outputs owned by the provided address to a remainder address of the account
    pub fn sweep_address_outputs<'a>(
        &'a self,
        address: Address,
        options: &'a Option<TransactionOptions>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<Transaction>> + Send + 'a>> {
        async move {
            let bech32_hrp = self.client().get_bech32_hrp().await?;
            let address = AddressWrapper::new(address, bech32_hrp);
            let remainder_address = self.get_sweep_remainder_address(&address, options).await?;

            let alias_outputs_state_controller = self.fetch_state_controller_address_alias_outputs(&address).await?;
            let alias_outputs_governor = self.fetch_governor_address_alias_outputs(&address).await?;
            // TODO: should we also check outputs with timelock, expiration and storage deposit return?
            let basic_outputs = self.fetch_address_basic_outputs(&address).await?;
            let foundry_outputs = self.fetch_foundry_outputs(&address).await?;
            // TODO: should we also check outputs with timelock, expiration and storage deposit return?
            let nft_outputs = self.fetch_address_nft_outputs(&address).await?;

            let mut output_ids = Vec::new();
            let mut outputs = Vec::new();

            let network_id = self.client.get_network_id().await?;

            for mut output_response in alias_outputs_state_controller
                .into_iter()
                .chain(alias_outputs_governor.into_iter())
                .chain(basic_outputs.into_iter())
                .chain(foundry_outputs.into_iter())
                .chain(nft_outputs.into_iter())
            {
                if output_response.metadata.is_spent {
                    continue;
                }

                let transaction_id = TransactionId::from_str(&output_response.metadata.transaction_id)?;
                let output_id = OutputId::new(transaction_id, output_response.metadata.output_index)?;
                self.update_unspent_output(output_response.clone(), output_id, network_id)
                    .await?;

                match &mut output_response.output {
                    OutputDto::Basic(output_dto) => {
                        replace_unlock_conditions(&mut output_dto.unlock_conditions, &remainder_address.inner);
                    }
                    OutputDto::Alias(alias_dto) => {
                        // TODO: check if alias doesn't own other outputs

                        // let mut alias_id: AliasId = alias_dto.alias_id.borrow().try_into()?;
                        // if alias_id.is_null() {
                        //     alias_id = AliasId::from(output_id);
                        // }
                        // // Recursively sweep alias address outputs
                        // let txn_ids = self
                        //     .sweep_address_outputs(Address::Alias(AliasAddress::new(alias_id)), options)
                        //     .await?;
                        // transaction_ids.extend(txn_ids);
                        replace_unlock_conditions(&mut alias_dto.unlock_conditions, &remainder_address.inner);
                    }
                    OutputDto::Nft(nft_dto) => {
                        // TODO: check if nft doesn't own other outputs

                        // let mut nft_id: NftId = nft_dto.nft_id.borrow().try_into()?;
                        // if nft_id.is_null() {
                        //     nft_id = NftId::from(output_id)
                        // }
                        // // Recursively sweep nft address outputs
                        // let txn_ids = self
                        //     .sweep_address_outputs(Address::Nft(NftAddress::new(nft_id)), options)
                        //     .await?;
                        // transaction_ids.extend(txn_ids);
                        replace_unlock_conditions(&mut nft_dto.unlock_conditions, &remainder_address.inner);
                    }
                    // Didn't ask for treasury and foundry outputs
                    OutputDto::Treasury(_) | OutputDto::Foundry(_) => continue,
                }

                let output = Output::try_from(&output_response.output)?;

                output_ids.push(output_id);
                outputs.push(output);

                // if output_ids.len() == (OUTPUT_COUNT_MAX - 1) as usize {
                //     let transaction_id = self
                //         .send_sweep_transaction(address.clone(), output_ids.drain(..), outputs.drain(..))
                //         .await?;
                //     transaction_ids.push(transaction_id);
                // }
            }

            if !output_ids.is_empty() {
                self.send_sweep_transaction(address.clone(), output_ids.drain(..), outputs.drain(..))
                    .await
            } else {
                Err(crate::Error::NoOutputsToConsolidate {
                    available_outputs: 0,
                    consolidation_threshold: 0,
                })
            }
        }
        .boxed()
    }

    pub(crate) async fn send_sweep_transaction(
        &self,
        address: AddressWrapper,
        output_ids: impl IntoIterator<Item = OutputId>,
        outputs: impl IntoIterator<Item = Output>,
    ) -> crate::Result<Transaction> {
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
                return Err(Error::BurningOrMeltingFailed(
                    "ed25519 address is not intended to be swept".to_string(),
                ));
            }
        }

        custom_inputs.append(&mut output_ids.into_iter().collect::<Vec<_>>());
        custom_outputs.append(&mut outputs.into_iter().collect::<Vec<_>>());

        let transaction_options = Some(TransactionOptions {
            custom_inputs: Some(custom_inputs),
            ..Default::default()
        });

        let transaction = self.send(custom_outputs, transaction_options).await?;
        match &transaction.block_id {
            Some(block_id) => {
                let _ = self.client.retry_until_included(block_id, None, None).await?;
                let _ = self.sync(None).await?;
            }
            None => {
                return Err(Error::BurningOrMeltingFailed(
                    "could not sweep address outputs".to_string(),
                ));
            }
        }

        Ok(transaction)
    }

    /// Fetches alias outputs with `address` set as Governor unlock condition
    pub(crate) async fn fetch_governor_address_alias_outputs(
        &self,
        address: &AddressWrapper,
    ) -> crate::Result<Vec<OutputResponse>> {
        let alias_query_parameters = vec![QueryParameter::Governor(address.to_bech32())];

        let alias_output_ids = self.client.alias_output_ids(alias_query_parameters).await?;
        let output_responses = self.client.get_outputs(alias_output_ids).await?;

        Ok(output_responses)
    }

    /// Fetches alias outputs with `address` set as StateController unlock condition
    pub(crate) async fn fetch_state_controller_address_alias_outputs(
        &self,
        address: &AddressWrapper,
    ) -> crate::Result<Vec<OutputResponse>> {
        let alias_query_parameters = vec![QueryParameter::StateController(address.to_bech32())];

        let alias_output_ids = self.client.alias_output_ids(alias_query_parameters).await?;
        let output_responses = self.client.get_outputs(alias_output_ids).await?;

        Ok(output_responses)
    }

    /// Fetches foundry outputs with `address` set as StateController unlock condition
    pub(crate) async fn fetch_foundry_outputs(&self, address: &AddressWrapper) -> crate::Result<Vec<OutputResponse>> {
        if let Address::Alias(_) = &address.inner {
        } else {
            return Ok(Vec::new());
        }
        let alias_query_parameters = vec![QueryParameter::StateController(address.to_bech32())];

        let alias_output_ids = self.client.alias_output_ids(alias_query_parameters).await?;
        let output_responses = self.client.get_outputs(alias_output_ids).await?;

        Ok(output_responses)
    }

    /// Fetches basic outputs with address unlock conditions only
    pub(crate) async fn fetch_address_basic_outputs(
        &self,
        address: &AddressWrapper,
    ) -> crate::Result<Vec<OutputResponse>> {
        let query_parameters = vec![
            QueryParameter::Address(address.to_bech32()),
            QueryParameter::HasExpiration(false),
            QueryParameter::HasTimelock(false),
            QueryParameter::HasStorageDepositReturn(false),
        ];

        let basic_output_ids = self.client.basic_output_ids(query_parameters.clone()).await?;
        let output_responses = self.client.get_outputs(basic_output_ids).await?;

        Ok(output_responses)
    }

    /// Fetches nft outputs with address unlock conditions only
    pub(crate) async fn fetch_address_nft_outputs(
        &self,
        address: &AddressWrapper,
    ) -> crate::Result<Vec<OutputResponse>> {
        let query_parameters = vec![
            QueryParameter::Address(address.to_bech32()),
            QueryParameter::HasExpiration(false),
            QueryParameter::HasTimelock(false),
            QueryParameter::HasStorageDepositReturn(false),
        ];

        let nfts_output_ids = self.client.nft_output_ids(query_parameters).await?;
        let output_responses = self.client.get_outputs(nfts_output_ids).await?;

        Ok(output_responses)
    }

    /// Update unspent output, this function is originally intended for updating recursively synced alias and nft
    /// address output
    async fn update_unspent_output(
        &self,
        output_response: OutputResponse,
        output_id: OutputId,
        network_id: u64,
    ) -> crate::Result<()> {
        let mut account = self.write().await;
        let output = Output::try_from(&output_response.output)?;
        let local_time = self.client.get_time_checked().await?;

        let (_amount, address) = ClientBlockBuilder::get_output_amount_and_address(&output, None, local_time)?;
        // check if we know the transaction that created this output and if we created it (if we store incoming
        // transactions separated, then this check wouldn't be required)
        let remainder = {
            match account.transactions.get(output_id.transaction_id()) {
                Some(tx) => !tx.incoming,
                None => false,
            }
        };

        let output_data = OutputData {
            output_id,
            output,
            is_spent: output_response.metadata.is_spent,
            metadata: output_response.metadata,
            address,
            network_id,
            remainder,
            chain: None,
        };

        account.unspent_outputs.entry(output_id).or_insert(output_data);

        Ok(())
    }
}

fn replace_unlock_conditions(unlock_conditions: &mut Vec<UnlockConditionDto>, address: &Address) {
    unlock_conditions.clear();
    unlock_conditions.push(UnlockConditionDto::Address(AddressUnlockConditionDto {
        kind: AddressUnlockCondition::KIND,
        address: address.into(),
    }));
}
