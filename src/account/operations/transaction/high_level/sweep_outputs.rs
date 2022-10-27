// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO: remove in the future: https://github.com/iotaledger/wallet.rs/issues/1453
#![allow(dead_code)]

use std::pin::Pin;

use futures::{Future, FutureExt};
use iota_client::{
    api::ClientBlockBuilder,
    api_types::response::OutputResponse,
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
    },
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
        let token_supply = self.client.get_token_supply()?;

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
            .finish(token_supply)?;

        Ok((output_id, new_state_alias_output))
    }

    // Enable in the future: https://github.com/iotaledger/wallet.rs/issues/1453
    // Will send outputs owned by the provided address to a remainder address of the account
    pub(crate) fn sweep_basic_outputs<'a>(
        &'a self,
        address: Address,
        options: &'a Option<TransactionOptions>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<Transaction>> + Send + 'a>> {
        async move {
            let token_supply = self.client.get_token_supply()?;
            let bech32_hrp = self.client().get_bech32_hrp()?;
            let address = AddressWrapper::new(address, bech32_hrp);
            let remainder_address = self.get_sweep_remainder_address(&address, options).await?;

            // TODO: get outputs from unspent outputs
            let basic_outputs: Vec<OutputResponse> = Vec::new();
            // maybe something like below
            // let mut outputs_to_sweep = Vec::new();
            // let mut other_owned_outputs = Vec::new();

            // for output_data in self.list_unspent_outputs(None).await? {
            //     // Ignore outputs with a single [UnlockCondition], because then it's an
            //     // [AddressUnlockCondition] and we own it already without
            //     // further restrictions
            //     if output_data
            //         .output
            //         .unlock_conditions()
            //         .expect("output needs to have unlock_conditions")
            //         .len()
            //         != 1
            //     {
            //         if can_output_be_unlocked_now(
            //             // We use the addresses with unspent outputs, because other addresses of the
            //             // account without unspent outputs can't be related to this output
            //             &addresses_with_unspent_outputs,
            //             &[Address::Nft(NftAddress::new(nft_id))],
            //             &output_data,
            //             current_time,
            //         ) {
            //             outputs_to_sweep.push(output_data);
            //         } else {
            //     // do we care about them?
            //             other_owned_outputs.push(output_data);
            //         }
            //     } else {
            //         outputs_to_sweep.push(output_data);
            //     }
            // }

            let mut output_ids = Vec::new();
            let mut outputs = Vec::new();

            let network_id = self.client.get_network_id()?;

            for mut output_response in basic_outputs {
                if output_response.metadata.is_spent {
                    continue;
                }

                let output_id = output_response.metadata.output_id()?;
                self.update_unspent_output(output_response.clone(), output_id, network_id)
                    .await?;

                // TODO don't mutate the output response, but use the output builder instead.
                match &mut output_response.output {
                    OutputDto::Basic(output_dto) => {
                        replace_unlock_conditions(&mut output_dto.unlock_conditions, &remainder_address.inner);
                    }
                    // Didn't ask for treasury and foundry outputs
                    OutputDto::Alias(_) | OutputDto::Nft(_) | OutputDto::Treasury(_) | OutputDto::Foundry(_) => {
                        continue;
                    }
                }

                let output = Output::try_from_dto(&output_response.output, token_supply)?;

                output_ids.push(output_id);
                outputs.push(output);
            }

            if !output_ids.is_empty() {
                self.send_sweep_transaction(address.clone(), output_ids.drain(..), outputs.drain(..))
                    .await
            } else {
                // TODO: Other error message?
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

    /// Update unspent output, this function is originally intended for updating recursively synced alias and nft
    /// address output
    async fn update_unspent_output(
        &self,
        output_response: OutputResponse,
        output_id: OutputId,
        network_id: u64,
    ) -> crate::Result<()> {
        let mut account = self.write().await;
        let token_supply = self.client.get_token_supply()?;
        let output = Output::try_from_dto(&output_response.output, token_supply)?;
        let local_time = self.client.get_time_checked()?;

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
