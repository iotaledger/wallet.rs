// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cmp::Ordering, str::FromStr};

use iota_client::bee_block::{
    address::Address,
    output::{
        dto::NativeTokenDto,
        feature::{Feature, MetadataFeature, TagFeature},
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition,
            TimelockUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder, ByteCost, NativeToken, NftId, Output,
    },
    payload::milestone::MilestoneIndex,
};
use serde::{Deserialize, Serialize};

use crate::account::{handle::AccountHandle, operations::transaction::RemainderValueStrategy, TransactionOptions};

impl AccountHandle {
    /// Prepare an output
    /// If the amount is below the minimum required storage deposit, by default the remaining amount will automatically
    /// added with a StorageDepositReturn UnlockCondition
    pub async fn prepare_output(
        &self,
        options: OutputOptions,
        transaction_options: Option<TransactionOptions>,
    ) -> crate::Result<Output> {
        log::debug!("[OUTPUT] prepare_output {options:?}");

        if let Some(assets) = &options.assets {
            if let Some(nft_id) = assets.nft_id {
                return self.prepare_nft_output(options.clone(), nft_id).await;
            }
        }
        let byte_cost_config = self.client.get_byte_cost_config().await?;

        // We start building with minimum storage deposit, so we know the minimum required amount and can later replace
        // it, if needed
        let mut basic_second_output_builder = BasicOutputBuilder::new_with_minimum_storage_deposit(
            byte_cost_config.clone(),
        )?
        .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
            Address::try_from_bech32(options.recipient_address.clone())?.1,
        )));

        if let Some(assets) = options.assets {
            if let Some(native_tokens) = assets.native_tokens {
                basic_second_output_builder = basic_second_output_builder.with_native_tokens(native_tokens);
            }
        }

        if let Some(features) = options.features {
            if let Some(tag) = features.tag {
                basic_second_output_builder = basic_second_output_builder
                    .add_feature(Feature::Tag(TagFeature::new(hex::encode(tag).as_bytes().to_vec())?));
            }
            if let Some(metadata) = features.metadata {
                basic_second_output_builder = basic_second_output_builder.add_feature(Feature::Metadata(
                    MetadataFeature::new(hex::encode(metadata).as_bytes().to_vec())?,
                ));
            }
        }

        if let Some(unlocks) = options.unlocks {
            if let Some(expiration) = unlocks.expiration {
                // todo: move this in an own method
                let remainder_address = match &transaction_options {
                    Some(options) => {
                        match &options.remainder_value_strategy {
                            RemainderValueStrategy::ReuseAddress => {
                                // select_inputs will select an address from the inputs if it's none
                                None
                            }
                            RemainderValueStrategy::ChangeAddress => {
                                let remainder_address = self.generate_remainder_address().await?;
                                Some(remainder_address.address().inner)
                            }
                            RemainderValueStrategy::CustomAddress(address) => Some(address.address().inner),
                        }
                    }
                    None => None,
                };
                let remainder_address = match remainder_address {
                    Some(address) => address,
                    None => self.generate_remainder_address().await?.address().inner,
                };

                basic_second_output_builder = basic_second_output_builder.add_unlock_condition(
                    UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                        remainder_address,
                        MilestoneIndex::new(expiration.milestone_index.unwrap_or_default()),
                        expiration.unix_time.unwrap_or_default(),
                    )?),
                );
            }
            if let Some(timelock) = unlocks.timelock {
                basic_second_output_builder = basic_second_output_builder.add_unlock_condition(
                    UnlockCondition::Timelock(TimelockUnlockCondition::new(
                        MilestoneIndex::new(timelock.milestone_index.unwrap_or_default()),
                        timelock.unix_time.unwrap_or_default(),
                    )?),
                );
            }
        }

        let first_output = basic_second_output_builder.finish()?;

        let mut second_output_builder = BasicOutputBuilder::from(&first_output);

        // Update the amount
        match options.amount.cmp(&first_output.amount()) {
            Ordering::Greater | Ordering::Equal => {
                // if it's larger than the minimum storage deposit, just replace it
                second_output_builder = second_output_builder.with_amount(options.amount)?;
            }
            Ordering::Less => {
                let storage_deposit = options.storage_deposit.unwrap_or_default();
                // Gift return strategy doesn't need a change, since the amount is already the minimum storage
                // deposit
                if let ReturnStrategy::Return = storage_deposit.return_strategy.unwrap_or_default() {
                    // todo: move this in an own method
                    let remainder_address = match &transaction_options {
                        Some(options) => {
                            match &options.remainder_value_strategy {
                                RemainderValueStrategy::ReuseAddress => {
                                    // select_inputs will select an address from the inputs if it's none
                                    None
                                }
                                RemainderValueStrategy::ChangeAddress => {
                                    let remainder_address = self.generate_remainder_address().await?;
                                    Some(remainder_address.address().inner)
                                }
                                RemainderValueStrategy::CustomAddress(address) => Some(address.address().inner),
                            }
                        }
                        None => None,
                    };
                    let remainder_address = match remainder_address {
                        Some(address) => address,
                        None => self.generate_remainder_address().await?.address().inner,
                    };

                    // Calculate the amount to be returned
                    let minimum_storage_deposit_remainder =
                        BasicOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config.clone())?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                Address::try_from_bech32(options.recipient_address)?.1,
                            )))
                            .finish_output()?
                            .amount();

                    second_output_builder = second_output_builder.add_unlock_condition(
                        UnlockCondition::StorageDepositReturn(StorageDepositReturnUnlockCondition::new(
                            remainder_address,
                            minimum_storage_deposit_remainder,
                        )?),
                    );
                }

                if storage_deposit.use_excess_if_low.unwrap_or_default() {
                    todo!(
                        "Get account balance and check if the remainding balance wouldn't leave dust behind, which wouldn't allow the creation of this output"
                    )
                }
            }
        }

        let second_output = second_output_builder.finish()?;

        let required_storage_deposit = Output::Basic(second_output.clone()).byte_cost(&byte_cost_config);

        let mut third_output_builder = BasicOutputBuilder::from(&second_output);

        // We might have added more unlock conditions, so we check the minimum storage deposit again and update the
        // amounts if needed
        if second_output.amount() < required_storage_deposit {
            third_output_builder = third_output_builder.with_amount(required_storage_deposit)?;
            // add newly added amount also to the storage deposit return unlock condition, if that was added
            let new_sdr_amount = required_storage_deposit - options.amount;
            if let Some(UnlockCondition::StorageDepositReturn(sdr)) = second_output
                .unlock_conditions()
                .get(StorageDepositReturnUnlockCondition::KIND)
            {
                // create a new sdr unlock_condition with the updated amount and replace it
                let new_sdr_unlock_condition = UnlockCondition::StorageDepositReturn(
                    StorageDepositReturnUnlockCondition::new(*sdr.return_address(), new_sdr_amount)?,
                );
                third_output_builder = third_output_builder.replace_unlock_condition(new_sdr_unlock_condition)?;
            }
        }

        // Build and return the final output
        Ok(third_output_builder.finish_output()?)
    }

    /// Prepare an nft output
    async fn prepare_nft_output(&self, options: OutputOptions, _nft_id: NftId) -> crate::Result<Output> {
        log::debug!("[OUTPUT] prepare_nft_output {options:?}");

        todo!("Implement prepare_nft_output");
        // check if nft output is available in account
        // add immutable fields
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputOptions {
    pub recipient_address: String,
    pub amount: u64,
    pub assets: Option<Assets>,
    pub features: Option<Features>,
    pub unlocks: Option<Unlocks>,
    pub storage_deposit: Option<StorageDeposit>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Assets {
    #[serde(rename = "nativeToken")]
    native_tokens: Option<Vec<NativeToken>>,
    #[serde(rename = "nftId")]
    nft_id: Option<NftId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Features {
    tag: Option<String>,
    metadata: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Unlocks {
    expiration: Option<Time>,
    timelock: Option<Time>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Time {
    #[serde(rename = "milestoneIndex")]
    milestone_index: Option<u32>,
    #[serde(rename = "unixTime")]
    unix_time: Option<u32>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageDeposit {
    #[serde(rename = "returnStrategy")]
    return_strategy: Option<ReturnStrategy>,
    // If account has 2 Mi, min storage deposit is 1 Mi and one wants to send 1.5 Mi, it wouldn't be possible with a
    // 0.5 Mi remainder. To still send a transaction, the 0.5 can be added to the output automatically, if set to true
    #[serde(rename = "useExcessIfLow")]
    use_excess_if_low: Option<bool>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReturnStrategy {
    // A storage deposit return unlock condition will be added with the required minimum storage deposit
    Return,
    // The recipient address will get the additional amount to reach the minimum storage deposit gifted
    Gift,
}

impl Default for ReturnStrategy {
    fn default() -> Self {
        ReturnStrategy::Return
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputOptionsDto {
    #[serde(rename = "recipientAddress")]
    recipient_address: String,
    amount: String,
    #[serde(default)]
    assets: Option<AssetsDto>,
    #[serde(default)]
    features: Option<Features>,
    #[serde(default)]
    unlocks: Option<Unlocks>,
    #[serde(rename = "storageDeposit", default)]
    storage_deposit: Option<StorageDeposit>,
}

impl TryFrom<&OutputOptionsDto> for OutputOptions {
    type Error = crate::Error;

    fn try_from(value: &OutputOptionsDto) -> crate::Result<Self> {
        Ok(Self {
            recipient_address: value.recipient_address.clone(),
            amount: u64::from_str(&value.amount)
                .map_err(|_| iota_client::Error::InvalidAmount(value.amount.clone()))?,
            assets: match &value.assets {
                Some(r) => Some(Assets::try_from(r)?),
                None => None,
            },
            features: value.features.clone(),
            unlocks: value.unlocks.clone(),
            storage_deposit: value.storage_deposit.clone(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetsDto {
    #[serde(rename = "nativeTokens")]
    native_tokens: Option<Vec<NativeTokenDto>>,
    #[serde(rename = "nftId")]
    nft_id: Option<String>,
}

impl TryFrom<&AssetsDto> for Assets {
    type Error = crate::Error;

    fn try_from(value: &AssetsDto) -> crate::Result<Self> {
        Ok(Self {
            native_tokens: match &value.native_tokens {
                Some(r) => Some(
                    r.iter()
                        .map(|r| Ok(NativeToken::try_from(r)?))
                        .collect::<crate::Result<Vec<NativeToken>>>()?,
                ),
                None => None,
            },
            nft_id: match &value.nft_id {
                Some(r) => Some(NftId::from_str(r)?),
                None => None,
            },
        })
    }
}
