// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{cmp::Ordering, str::FromStr};

use iota_client::block::{
    address::Address,
    output::{
        dto::NativeTokenDto,
        feature::{Feature, MetadataFeature, TagFeature, SenderFeature, IssuerFeature},
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition,
            TimelockUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder, NativeToken, NftId, NftOutputBuilder, Output, Rent,
    },
};
use serde::{Deserialize, Serialize};

use crate::account::{handle::AccountHandle, operations::transaction::RemainderValueStrategy, TransactionOptions};

impl AccountHandle {
    /// Prepare an output for sending
    /// If the amount is below the minimum required storage deposit, by default the remaining amount will automatically
    /// be added with a StorageDepositReturn UnlockCondition, when setting the ReturnStrategy to `gift`, the full
    /// minimum required storage deposit will be sent to the recipient.
    /// When the assets contain an nft_id, the data from the existing nft output will be used, just with the address
    /// unlock conditions replaced
    pub async fn prepare_output(
        &self,
        options: OutputOptions,
        transaction_options: Option<TransactionOptions>,
    ) -> crate::Result<Output> {
        log::debug!("[OUTPUT] prepare_output {options:?}");
        let token_supply = self.client.get_token_supply().await?;

        if let Some(assets) = &options.assets {
            if let Some(nft_id) = assets.nft_id {
                return self
                    .prepare_nft_output(options.clone(), transaction_options, nft_id)
                    .await;
            }
        }
        let rent_structure = self.client.get_rent_structure().await?;

        // We start building with minimum storage deposit, so we know the minimum required amount and can later replace
        // it, if needed
        let mut first_output_builder = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure.clone())?
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                Address::try_from_bech32(options.recipient_address.clone())?.1,
            )));

        if let Some(assets) = options.assets {
            if let Some(native_tokens) = assets.native_tokens {
                first_output_builder = first_output_builder.with_native_tokens(native_tokens);
            }
        }

        if let Some(features) = options.features {
            if let Some(tag) = features.tag {
                first_output_builder =
                    first_output_builder.add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?));
            }
            if let Some(metadata) = features.metadata {
                first_output_builder = first_output_builder
                    .add_feature(Feature::Metadata(MetadataFeature::new(metadata.as_bytes().to_vec())?));
            }

            if let Some(issuer) = features.issuer {
                first_output_builder = first_output_builder
                    .add_feature(Feature::Issuer(IssuerFeature::new(Address::try_from_bech32(&issuer)?.1)));
            }

            if let Some(sender) = features.sender {
                first_output_builder = first_output_builder
                    .add_feature(Feature::Sender(SenderFeature::new(Address::try_from_bech32(&sender)?.1)))
            }
        }

        if let Some(unlocks) = options.unlocks {
            if let Some(expiration_unix_time) = unlocks.expiration_unix_time {
                let remainder_address = self.get_remainder_address(transaction_options.clone()).await?;

                first_output_builder = first_output_builder.add_unlock_condition(UnlockCondition::Expiration(
                    ExpirationUnlockCondition::new(remainder_address, expiration_unix_time)?,
                ));
            }
            if let Some(timelock_unix_time) = unlocks.timelock_unix_time {
                first_output_builder = first_output_builder.add_unlock_condition(UnlockCondition::Timelock(
                    TimelockUnlockCondition::new(timelock_unix_time)?,
                ));
            }
        }

        let first_output = first_output_builder.finish(token_supply)?;

        let mut second_output_builder = BasicOutputBuilder::from(&first_output);

        let mut min_storage_deposit_return_amount = 0;
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
                    let remainder_address = self.get_remainder_address(transaction_options).await?;

                    // Calculate the minimum storage deposit to be returned
                    min_storage_deposit_return_amount =
                        BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure.clone())?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                Address::try_from_bech32(options.recipient_address.clone())?.1,
                            )))
                            .finish_output(token_supply)?
                            .amount();

                    second_output_builder = second_output_builder.add_unlock_condition(
                        UnlockCondition::StorageDepositReturn(StorageDepositReturnUnlockCondition::new(
                            remainder_address,
                            // Return minimum storage deposit + any additional required storage deposit from features
                            // or unlock conditions
                            min_storage_deposit_return_amount,
                            token_supply,
                        )?),
                    );
                }

                // Check if the remainder balance wouldn't leave dust behind, which wouldn't allow the creation of this
                // output. If that's the case, this remaining amount will be added to the output, to still allow sending
                // it.
                if storage_deposit.use_excess_if_low.unwrap_or_default() {
                    let balance = self.balance().await?;
                    if let Ordering::Greater = balance.base_coin.available.cmp(&first_output.amount()) {
                        let balance_minus_output = balance.base_coin.available - first_output.amount();
                        // Calculate the amount for a basic output
                        let minimum_required_storage_deposit =
                            BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure.clone())?
                                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                    Address::try_from_bech32(options.recipient_address)?.1,
                                )))
                                .finish_output(token_supply)?
                                .amount();

                        if balance_minus_output < minimum_required_storage_deposit {
                            second_output_builder =
                                second_output_builder.with_amount(first_output.amount() + balance_minus_output)?;
                        }
                    }
                }
            }
        }

        let second_output = second_output_builder.finish(token_supply)?;

        let required_storage_deposit = Output::Basic(second_output.clone()).rent_cost(&rent_structure);

        let mut third_output_builder = BasicOutputBuilder::from(&second_output);

        // We might have added more unlock conditions, so we check the minimum storage deposit again and update the
        // amounts if needed
        if second_output.amount() < required_storage_deposit {
            third_output_builder = third_output_builder.with_amount(required_storage_deposit)?;
            // add newly added amount also to the storage deposit return unlock condition, if that was added
            let mut new_sdr_amount = required_storage_deposit - options.amount;
            // If the new sdr amount is lower than it needs to be, set it to the minimum
            if new_sdr_amount < min_storage_deposit_return_amount {
                new_sdr_amount = min_storage_deposit_return_amount;
                third_output_builder =
                    third_output_builder.with_amount(min_storage_deposit_return_amount + options.amount)?;
            }
            if let Some(sdr) = second_output.unlock_conditions().storage_deposit_return() {
                // create a new sdr unlock_condition with the updated amount and replace it
                let new_sdr_unlock_condition = UnlockCondition::StorageDepositReturn(
                    StorageDepositReturnUnlockCondition::new(*sdr.return_address(), new_sdr_amount, token_supply)?,
                );
                third_output_builder = third_output_builder.replace_unlock_condition(new_sdr_unlock_condition)?;
            }
        }

        // Build and return the final output
        Ok(third_output_builder.finish_output(token_supply)?)
    }

    /// Prepare an nft output
    async fn prepare_nft_output(
        &self,
        options: OutputOptions,
        transaction_options: Option<TransactionOptions>,
        nft_id: NftId,
    ) -> crate::Result<Output> {
        log::debug!("[OUTPUT] prepare_nft_output {options:?}");

        let token_supply = self.client.get_token_supply().await?;
        let unspent_outputs = self.unspent_outputs(None).await?;

        // Find nft output from the inputs
        let mut first_output_builder = if let Some(nft_output_data) = unspent_outputs.iter().find(|o| {
            if let Output::Nft(nft_output) = &o.output {
                nft_id == nft_output.nft_id().or_from_output_id(o.output_id)
            } else {
                false
            }
        }) {
            if let Output::Nft(nft_output) = &nft_output_data.output {
                NftOutputBuilder::from(nft_output)
            } else {
                unreachable!("We checked before if it's an nft output")
            }
        } else {
            return Err(crate::Error::NftNotFoundInUnspentOutputs);
        };

        // Set new address unlock condition
        first_output_builder = first_output_builder.with_unlock_conditions(vec![UnlockCondition::Address(
            AddressUnlockCondition::new(Address::try_from_bech32(options.recipient_address.clone())?.1),
        )]);

        // from here basically the same as in `prepare_output()`, just with Nft outputs

        let rent_structure = self.client.get_rent_structure().await?;

        if let Some(assets) = options.assets {
            if let Some(native_tokens) = assets.native_tokens {
                first_output_builder = first_output_builder.with_native_tokens(native_tokens);
            }
        }

        if let Some(features) = options.features {
            if let Some(tag) = features.tag {
                first_output_builder =
                    first_output_builder.add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?));
            }
            if let Some(metadata) = features.metadata {
                first_output_builder = first_output_builder
                    .add_feature(Feature::Metadata(MetadataFeature::new(metadata.as_bytes().to_vec())?));
            }

            if let Some(issuer) = features.issuer {
                first_output_builder = first_output_builder
                    .add_feature(Feature::Issuer(IssuerFeature::new(Address::try_from_bech32(&issuer)?.1)));
            }

            if let Some(sender) = features.sender {
                first_output_builder = first_output_builder
                    .add_feature(Feature::Sender(SenderFeature::new(Address::try_from_bech32(&sender)?.1)))
            }
        }

        if let Some(unlocks) = options.unlocks {
            if let Some(expiration_unix_time) = unlocks.expiration_unix_time {
                let remainder_address = self.get_remainder_address(transaction_options.clone()).await?;

                first_output_builder = first_output_builder.add_unlock_condition(UnlockCondition::Expiration(
                    ExpirationUnlockCondition::new(remainder_address, expiration_unix_time)?,
                ));
            }
            if let Some(timelock_unix_time) = unlocks.timelock_unix_time {
                first_output_builder = first_output_builder.add_unlock_condition(UnlockCondition::Timelock(
                    TimelockUnlockCondition::new(timelock_unix_time)?,
                ));
            }
        }

        let first_output = first_output_builder.finish(token_supply)?;

        let mut second_output_builder = NftOutputBuilder::from(&first_output);

        let mut min_storage_deposit_return_amount = 0;
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
                    let remainder_address = self.get_remainder_address(transaction_options).await?;

                    // Calculate the amount to be returned
                    min_storage_deposit_return_amount =
                        BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure.clone())?
                            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                Address::try_from_bech32(options.recipient_address.clone())?.1,
                            )))
                            .finish_output(token_supply)?
                            .amount();

                    second_output_builder = second_output_builder.add_unlock_condition(
                        UnlockCondition::StorageDepositReturn(StorageDepositReturnUnlockCondition::new(
                            remainder_address,
                            // Return minimum storage deposit + any additional required storage deposit from features
                            // or unlock conditions
                            min_storage_deposit_return_amount,
                            token_supply,
                        )?),
                    );
                }

                // Check if the remainding balance wouldn't leave dust behind, which wouldn't allow the creation of this
                // output. If that's the case, this remaining amount will be added to the output, to still allow sending
                // it.
                if storage_deposit.use_excess_if_low.unwrap_or_default() {
                    let balance = self.balance().await?;
                    if let Ordering::Greater = balance.base_coin.available.cmp(&first_output.amount()) {
                        let balance_minus_output = balance.base_coin.available - first_output.amount();
                        // Calculate the amount for a basic output
                        let minimum_required_storage_deposit =
                            BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure.clone())?
                                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                                    Address::try_from_bech32(options.recipient_address)?.1,
                                )))
                                .finish_output(token_supply)?
                                .amount();

                        if balance_minus_output < minimum_required_storage_deposit {
                            second_output_builder =
                                second_output_builder.with_amount(first_output.amount() + balance_minus_output)?;
                        }
                    }
                }
            }
        }

        let second_output = second_output_builder.finish(token_supply)?;

        let required_storage_deposit = Output::Nft(second_output.clone()).rent_cost(&rent_structure);

        let mut third_output_builder = NftOutputBuilder::from(&second_output);

        // We might have added more unlock conditions, so we check the minimum storage deposit again and update the
        // amounts if needed
        if second_output.amount() < required_storage_deposit {
            let mut new_sdr_amount = required_storage_deposit - options.amount;
            // If the new sdr amount is lower than it needs to be, set it to the minimum
            if new_sdr_amount < min_storage_deposit_return_amount {
                new_sdr_amount = min_storage_deposit_return_amount;
                third_output_builder =
                    third_output_builder.with_amount(min_storage_deposit_return_amount + options.amount)?;
            }
            if let Some(sdr) = second_output.unlock_conditions().storage_deposit_return() {
                // create a new sdr unlock_condition with the updated amount and replace it
                let new_sdr_unlock_condition = UnlockCondition::StorageDepositReturn(
                    StorageDepositReturnUnlockCondition::new(*sdr.return_address(), new_sdr_amount, token_supply)?,
                );
                third_output_builder = third_output_builder.replace_unlock_condition(new_sdr_unlock_condition)?;
            }
        }

        // Build and return the final output
        Ok(third_output_builder.finish_output(token_supply)?)
    }

    // Get a remainder address based on transaction_options or use the first account address
    async fn get_remainder_address(&self, transaction_options: Option<TransactionOptions>) -> crate::Result<Address> {
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
            None => {
                self.addresses()
                    .await?
                    .first()
                    .ok_or(crate::Error::FailedToGetRemainder)?
                    .address()
                    .inner
            }
        };
        Ok(remainder_address)
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
    pub native_tokens: Option<Vec<NativeToken>>,
    #[serde(rename = "nftId")]
    pub nft_id: Option<NftId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Features {
    pub tag: Option<String>,
    pub metadata: Option<String>,
    pub issuer: Option<String>,
    pub sender: Option<String>
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Unlocks {
    #[serde(rename = "expirationUnixTime")]
    pub expiration_unix_time: Option<u32>,
    #[serde(rename = "timelockUnixTime")]
    pub timelock_unix_time: Option<u32>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageDeposit {
    #[serde(rename = "returnStrategy")]
    pub return_strategy: Option<ReturnStrategy>,
    // If account has 2 Mi, min storage deposit is 1 Mi and one wants to send 1.5 Mi, it wouldn't be possible with a
    // 0.5 Mi remainder. To still send a transaction, the 0.5 can be added to the output automatically, if set to true
    #[serde(rename = "useExcessIfLow")]
    pub use_excess_if_low: Option<bool>,
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
