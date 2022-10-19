// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Dtos with amount as String, to prevent overflow issues in other languages

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use iota_client::{
    api_types::response::OutputResponse,
    block::{
        dto::U256Dto,
        output::{dto::TokenIdDto, AliasId, FoundryId, NftId, OutputId},
        payload::transaction::{dto::TransactionPayloadDto, TransactionId},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        types::{
            address::AddressWrapper, AccountAddress, AccountBalance, AddressWithUnspentOutputs, BaseCoinBalance,
            NativeTokensBalance, TransactionDto,
        },
        Account, OutputDataDto,
    },
    AddressWithAmount, AddressWithMicroAmount,
};

/// Dto for address with amount for `send_amount()`
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AddressWithAmountDto {
    /// Bech32 encoded address
    pub address: String,
    /// Amount
    pub amount: String,
}

impl TryFrom<&AddressWithAmountDto> for AddressWithAmount {
    type Error = crate::Error;

    fn try_from(value: &AddressWithAmountDto) -> crate::Result<Self> {
        Ok(Self {
            address: value.address.clone(),
            amount: u64::from_str(&value.amount)
                .map_err(|_| iota_client::Error::InvalidAmount(value.amount.clone()))?,
        })
    }
}

/// Dto for address with amount for `send_micro_transaction()`
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AddressWithMicroAmountDto {
    /// Bech32 encoded address
    pub address: String,
    /// Amount below the minimum storage deposit
    pub amount: String,
    /// Bech32 encoded address return address, to which the storage deposit will be returned. Default will use the
    /// first address of the account
    #[serde(rename = "returnAddress")]
    pub return_address: Option<String>,
    /// Expiration in seconds, after which the output will be available for the sender again, if not spent by the
    /// receiver before. Default is 1 day
    pub expiration: Option<u32>,
}

impl TryFrom<&AddressWithMicroAmountDto> for AddressWithMicroAmount {
    type Error = crate::Error;

    fn try_from(value: &AddressWithMicroAmountDto) -> crate::Result<Self> {
        Ok(Self {
            address: value.address.clone(),
            amount: u64::from_str(&value.amount)
                .map_err(|_| iota_client::Error::InvalidAmount(value.amount.clone()))?,
            return_address: value.return_address.clone(),
            expiration: value.expiration,
        })
    }
}

/// Dto for an account address with output_ids of unspent outputs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AddressWithUnspentOutputsDto {
    /// The address.
    #[serde(with = "crate::account::types::address_serde")]
    pub address: AddressWrapper,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    pub key_index: u32,
    /// Determines if an address is a public or an internal (change) address.
    pub internal: bool,
    /// Output ids
    #[serde(rename = "outputIds")]
    pub output_ids: Vec<OutputId>,
}

impl From<&AddressWithUnspentOutputs> for AddressWithUnspentOutputsDto {
    fn from(value: &AddressWithUnspentOutputs) -> Self {
        Self {
            address: value.address.clone(),
            key_index: value.key_index,
            internal: value.internal,
            output_ids: value.output_ids.clone(),
        }
    }
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

/// Base coin fields for [`AccountBalanceDto`]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeTokensBalanceDto {
    /// Token id
    #[serde(rename = "tokenId")]
    pub token_id: TokenIdDto,
    /// Total amount
    pub total: U256Dto,
    /// Balance that can currently be spent
    pub available: U256Dto,
}

impl From<&NativeTokensBalance> for NativeTokensBalanceDto {
    fn from(value: &NativeTokensBalance) -> Self {
        Self {
            token_id: TokenIdDto::from(&value.token_id),
            total: U256Dto::from(&value.total),
            available: U256Dto::from(&value.available),
        }
    }
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
    pub required_storage_deposit: String,
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
            required_storage_deposit: value.required_storage_deposit.to_string(),
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

/// Dto for an Account.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountDto {
    /// The account index
    pub index: u32,
    /// The coin type
    #[serde(rename = "coinType")]
    pub coin_type: u32,
    /// The account alias.
    pub alias: String,
    /// Public addresses
    #[serde(rename = "publicAddresses")]
    pub public_addresses: Vec<AccountAddress>,
    /// Internal addresses
    #[serde(rename = "internalAddresses")]
    pub internal_addresses: Vec<AccountAddress>,
    /// Addresses with unspent outputs
    #[serde(rename = "addressesWithUnspentOutputs")]
    pub addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputsDto>,
    /// Outputs
    pub outputs: HashMap<OutputId, OutputDataDto>,
    /// Unspent outputs that are currently used as input for transactions
    #[serde(rename = "lockedOutputs")]
    pub locked_outputs: HashSet<OutputId>,
    /// Unspent outputs
    #[serde(rename = "unspentOutputs")]
    pub unspent_outputs: HashMap<OutputId, OutputDataDto>,
    /// Sent transactions
    pub transactions: HashMap<TransactionId, TransactionDto>,
    /// Pending transactions
    #[serde(rename = "pendingTransactions")]
    pub pending_transactions: HashSet<TransactionId>,
    /// Incoming transactions
    #[serde(rename = "incomingTransactions")]
    pub incoming_transactions: HashMap<TransactionId, (TransactionPayloadDto, Vec<OutputResponse>)>,
}

impl From<&Account> for AccountDto {
    fn from(value: &Account) -> Self {
        Self {
            index: *value.index(),
            coin_type: *value.coin_type(),
            alias: value.alias().clone(),
            public_addresses: value.public_addresses.clone(),
            internal_addresses: value.internal_addresses.clone(),
            addresses_with_unspent_outputs: value
                .addresses_with_unspent_outputs()
                .iter()
                .map(AddressWithUnspentOutputsDto::from)
                .collect(),
            outputs: value
                .outputs()
                .clone()
                .into_iter()
                .map(|(k, o)| (k, OutputDataDto::from(&o)))
                .collect(),
            locked_outputs: value.locked_outputs().clone(),
            unspent_outputs: value
                .unspent_outputs()
                .clone()
                .into_iter()
                .map(|(k, o)| (k, OutputDataDto::from(&o)))
                .collect(),
            transactions: value
                .transactions()
                .clone()
                .into_iter()
                .map(|(k, o)| (k, TransactionDto::from(&o)))
                .collect(),
            pending_transactions: value.pending_transactions().clone(),
            incoming_transactions: value
                .incoming_transactions()
                .iter()
                .map(|(tx_id, (tx, inputs))| (*tx_id, (TransactionPayloadDto::from(tx), inputs.clone())))
                .collect(),
        }
    }
}
