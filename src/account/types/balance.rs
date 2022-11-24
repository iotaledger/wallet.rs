// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::block::output::{AliasId, FoundryId, NftId, OutputId, TokenId};
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
    pub required_storage_deposit: u64,
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

/// Base coin fields for [`AccountBalance`]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaseCoinBalance {
    /// Total amount
    pub total: u64,
    /// Balance that can currently be spent
    pub available: u64,
}

/// Native tokens fields for [`AccountBalance`]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeTokensBalance {
    /// Token id
    #[serde(rename = "tokenId")]
    pub token_id: TokenId,
    /// Total amount
    pub total: U256,
    /// Balance that can currently be spent
    pub available: U256,
}

impl Default for NativeTokensBalance {
    fn default() -> Self {
        Self {
            token_id: TokenId::null(),
            total: U256::from(0u8),
            available: U256::from(0u8),
        }
    }
}
