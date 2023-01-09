// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::block::{
    dto::U256Dto,
    output::{dto::TokenIdDto, feature::MetadataFeature, AliasId, FoundryId, NftId, OutputId, TokenId},
};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// The balance of an account, returned from [`crate::account::handle::AccountHandle::sync()`] and
/// [`crate::account::handle::AccountHandle::balance()`].
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Total and available amount of the base coin
    #[serde(rename = "baseCoin")]
    pub base_coin: BaseCoinBalance,
    /// Current required storage deposit amount
    #[serde(rename = "requiredStorageDeposit")]
    pub required_storage_deposit: RequiredStorageDeposit,
    /// Native tokens
    #[serde(rename = "nativeTokens")]
    pub native_tokens: Vec<NativeTokensBalance>,
    /// Nfts
    pub nfts: Vec<NftId>,
    /// Aliases
    pub aliases: Vec<AliasId>,
    /// Foundries
    pub foundries: Vec<FoundryId>,
    /// Outputs with multiple unlock conditions and if they can currently be spent or not. If there is a
    /// [`TimelockUnlockCondition`](iota_client::block::output::unlock_condition::TimelockUnlockCondition) or
    /// [`ExpirationUnlockCondition`](iota_client::block::output::unlock_condition::ExpirationUnlockCondition) this can
    /// change at any time
    #[serde(rename = "potentiallyLockedOutputs")]
    pub potentially_locked_outputs: HashMap<OutputId, bool>,
}

/// Dto for the balance of an account, returned from [`crate::account::handle::AccountHandle::sync()`] and
/// [`crate::account::handle::AccountHandle::balance()`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountBalanceDto {
    /// Total and available amount of the base coin
    #[serde(rename = "baseCoin")]
    pub base_coin: BaseCoinBalanceDto,
    /// Current required storage deposit amount
    #[serde(rename = "requiredStorageDeposit")]
    pub required_storage_deposit: RequiredStorageDepositDto,
    /// Native tokens
    #[serde(rename = "nativeTokens")]
    pub native_tokens: Vec<NativeTokensBalanceDto>,
    /// Nfts
    pub nfts: Vec<NftId>,
    /// Aliases
    pub aliases: Vec<AliasId>,
    /// Foundries
    pub foundries: Vec<FoundryId>,
    /// Outputs with multiple unlock conditions and if they can currently be spent or not. If there is a
    /// [`TimelockUnlockCondition`] or [`ExpirationUnlockCondition`] this can change at any time
    #[serde(rename = "potentiallyLockedOutputs")]
    pub potentially_locked_outputs: HashMap<OutputId, bool>,
}

impl From<&AccountBalance> for AccountBalanceDto {
    fn from(value: &AccountBalance) -> Self {
        Self {
            base_coin: BaseCoinBalanceDto::from(&value.base_coin),
            required_storage_deposit: RequiredStorageDepositDto::from(&value.required_storage_deposit),
            native_tokens: value
                .native_tokens
                .iter()
                .map(NativeTokensBalanceDto::from)
                .collect::<_>(),
            nfts: value.nfts.clone(),
            aliases: value.aliases.clone(),
            foundries: value.foundries.clone(),
            potentially_locked_outputs: value.potentially_locked_outputs.clone(),
        }
    }
}

/// Base coin fields for [`AccountBalance`]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaseCoinBalance {
    /// Total amount
    pub total: u64,
    /// Balance that can currently be spent
    pub available: u64,
}

/// Base coin fields for [`AccountBalance`]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaseCoinBalanceDto {
    /// Total amount
    pub total: String,
    /// Balance that can currently be spent
    pub available: String,
}

impl From<&BaseCoinBalance> for BaseCoinBalanceDto {
    fn from(value: &BaseCoinBalance) -> Self {
        Self {
            total: value.total.to_string(),
            available: value.available.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequiredStorageDeposit {
    pub(crate) alias: u64,
    pub(crate) basic: u64,
    pub(crate) foundry: u64,
    pub(crate) nft: u64,
}

impl RequiredStorageDeposit {
    pub fn new() -> RequiredStorageDeposit {
        RequiredStorageDeposit::default()
    }

    pub fn alias(&self) -> u64 {
        self.alias
    }

    pub fn basic(&self) -> u64 {
        self.basic
    }

    pub fn foundry(&self) -> u64 {
        self.foundry
    }

    pub fn nft(&self) -> u64 {
        self.nft
    }
}

impl std::ops::AddAssign for RequiredStorageDeposit {
    fn add_assign(&mut self, rhs: RequiredStorageDeposit) {
        self.alias += rhs.alias;
        self.basic += rhs.basic;
        self.foundry += rhs.foundry;
        self.nft += rhs.nft;
    }
}

/// DTO for [`RequiredStorageDeposit`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequiredStorageDepositDto {
    pub(crate) alias: String,
    pub(crate) basic: String,
    pub(crate) foundry: String,
    pub(crate) nft: String,
}

impl From<&RequiredStorageDeposit> for RequiredStorageDepositDto {
    fn from(value: &RequiredStorageDeposit) -> Self {
        Self {
            alias: value.alias.to_string(),
            basic: value.basic.to_string(),
            foundry: value.foundry.to_string(),
            nft: value.nft.to_string(),
        }
    }
}

/// Native tokens fields for [`AccountBalance`]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeTokensBalance {
    /// Token id
    #[serde(rename = "tokenId")]
    pub token_id: TokenId,
    /// Token foundry immutable metadata
    pub metadata: Option<MetadataFeature>,
    /// Total amount
    pub total: U256,
    /// Balance that can currently be spent
    pub available: U256,
}

impl Default for NativeTokensBalance {
    fn default() -> Self {
        Self {
            token_id: TokenId::null(),
            metadata: None,
            total: U256::from(0u8),
            available: U256::from(0u8),
        }
    }
}

/// Base coin fields for [`AccountBalanceDto`]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeTokensBalanceDto {
    /// Token id
    #[serde(rename = "tokenId")]
    pub token_id: TokenIdDto,
    /// Token foundry immutable metadata
    pub metadata: Option<String>,
    /// Total amount
    pub total: U256Dto,
    /// Balance that can currently be spent
    pub available: U256Dto,
}

impl From<&NativeTokensBalance> for NativeTokensBalanceDto {
    fn from(value: &NativeTokensBalance) -> Self {
        Self {
            token_id: TokenIdDto::from(&value.token_id),
            metadata: value.metadata.as_ref().map(|m| prefix_hex::encode(m.data())),
            total: U256Dto::from(&value.total),
            available: U256Dto::from(&value.available),
        }
    }
}
