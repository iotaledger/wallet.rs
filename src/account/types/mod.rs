// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Address types used in the account
pub(crate) mod address;
pub use address::{AccountAddress, AddressWithUnspentOutputs};
/// Custom de/serialization for [`address::AddressWrapper`]
pub(crate) mod address_serde;

use std::{collections::HashMap, str::FromStr};

use crypto::keys::slip10::Chain;
use iota_client::{
    api_types::response::OutputMetadataResponse,
    block::{
        address::{dto::AddressDto, Address},
        output::{dto::OutputDto, AliasId, FoundryId, NftId, Output, OutputId, TokenId},
        payload::transaction::{dto::TransactionPayloadDto, TransactionId, TransactionPayload},
        BlockId,
    },
    secret::types::{InputSigningData, OutputMetadata},
};
use primitive_types::U256;
use serde::{Deserialize, Deserializer, Serialize};

use crate::account::Account;

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
    /// [`TimelockUnlockCondition`] or [`ExpirationUnlockCondition`] this can change at any time
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

/// An output with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputData {
    /// The output id
    #[serde(rename = "outputId")]
    pub output_id: OutputId,
    pub metadata: OutputMetadataResponse,
    /// The actual Output
    pub output: Output,
    /// If an output is spent
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    /// Associated account address.
    pub address: Address,
    /// Network ID
    #[serde(rename = "networkId")]
    pub network_id: u64,
    pub remainder: bool,
    // bip32 path
    pub chain: Option<Chain>,
}

impl OutputData {
    pub fn input_signing_data(
        &self,
        account: &Account,
        current_time: u32,
        bech32_hrp: &str,
    ) -> crate::Result<InputSigningData> {
        let unlock_address = {
            if let Some(unlock_conditions) = self.output.unlock_conditions() {
                match &self.output {
                    Output::Foundry(foundry) => Address::Alias(*foundry.alias_address()),
                    _ => *unlock_conditions.locked_address(&self.address, current_time),
                }
            } else {
                self.address
            }
        };

        let chain = if unlock_address == self.address {
            self.chain.clone()
        } else if let Address::Ed25519(_) = unlock_address {
            if let Some(address) = account
                .addresses_with_unspent_outputs
                .iter()
                .find(|a| a.address.inner == unlock_address)
            {
                Some(Chain::from_u32_hardened(vec![
                    44,
                    account.coin_type,
                    account.index,
                    address.internal as u32,
                    address.key_index,
                ]))
            } else {
                return Err(crate::Error::CustomInputError(
                    "unlock address not found in account addresses".to_string(),
                ));
            }
        } else {
            // Alias and NFT addresses have no chain
            None
        };

        Ok(InputSigningData {
            output: self.output.clone(),
            output_metadata: OutputMetadata::try_from(&self.metadata)?,
            chain,
            bech32_address: unlock_address.to_bech32(bech32_hrp),
        })
    }
}

/// Dto for an output with metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputDataDto {
    /// The output id
    #[serde(rename = "outputId")]
    pub output_id: OutputId,
    /// The metadata of the output
    pub metadata: OutputMetadataResponse,
    /// The actual Output
    pub output: OutputDto,
    /// If an output is spent
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    /// Associated account address.
    pub address: AddressDto,
    /// Network ID
    #[serde(rename = "networkId")]
    pub network_id: String,
    /// Remainder
    pub remainder: bool,
    /// Bip32 path
    pub chain: Option<Chain>,
}

impl From<&OutputData> for OutputDataDto {
    fn from(value: &OutputData) -> Self {
        Self {
            output_id: value.output_id,
            metadata: value.metadata.clone(),
            output: OutputDto::from(&value.output),
            is_spent: value.is_spent,
            address: AddressDto::from(&value.address),
            network_id: value.network_id.to_string(),
            remainder: value.remainder,
            chain: value.chain.clone(),
        }
    }
}

/// A transaction with metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub payload: TransactionPayload,
    #[serde(rename = "blockId")]
    pub block_id: Option<BlockId>,
    #[serde(rename = "inclusionState")]
    pub inclusion_state: InclusionState,
    // Transaction creation time
    pub timestamp: u128,
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId,
    // network id to ignore outputs when set_client_options is used to switch to another network
    #[serde(rename = "networkId")]
    pub network_id: u64,
    // set if the transaction was created by the wallet or if it was sent by someone else and is incoming
    pub incoming: bool,
    pub note: Option<String>,
}

/// Dto for a transaction with metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionDto {
    /// The transaction payload
    pub payload: TransactionPayloadDto,
    /// BlockId when it got sent to the Tangle
    #[serde(rename = "blockId")]
    pub block_id: Option<BlockId>,
    /// Inclusion state of the transaction
    #[serde(rename = "inclusionState")]
    pub inclusion_state: InclusionState,
    /// Timestamp
    pub timestamp: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId,
    /// Network id to ignore outputs when set_client_options is used to switch to another network
    #[serde(rename = "networkId")]
    pub network_id: String,
    /// If the transaction was created by the wallet or if it was sent by someone else and is incoming
    pub incoming: bool,
    pub note: Option<String>,
}

impl From<&Transaction> for TransactionDto {
    fn from(value: &Transaction) -> Self {
        Self {
            payload: TransactionPayloadDto::from(&value.payload),
            block_id: value.block_id,
            inclusion_state: value.inclusion_state,
            timestamp: value.timestamp.to_string(),
            transaction_id: value.transaction_id,
            network_id: value.network_id.to_string(),
            incoming: value.incoming,
            note: value.note.clone(),
        }
    }
}

/// Possible InclusionStates for transactions
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InclusionState {
    Pending,
    Confirmed,
    Conflicting,
    UnknownPruned,
}

/// The output kind enum.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum OutputKind {
    /// Alias output.
    Alias,
    /// Basic output.
    Basic,
    /// Foundry output.
    Foundry,
    /// Nft output.
    Nft,
    /// Treasury output.
    Treasury,
}

impl FromStr for OutputKind {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "Alias" => Self::Alias,
            "Basic" => Self::Basic,
            "Foundry" => Self::Foundry,
            "Nft" => Self::Nft,
            "Treasury" => Self::Treasury,
            _ => return Err(crate::Error::InvalidOutputKind(s.to_string())),
        };
        Ok(kind)
    }
}

/// The account identifier.
#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum AccountIdentifier {
    /// Account alias as identifier.
    Alias(String),
    /// An index identifier.
    Index(u32),
}

// Custom deserialize because the index could also be encoded as String
impl<'de> Deserialize<'de> for AccountIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;
        let v = Value::deserialize(deserializer)?;
        Ok(match v.as_u64() {
            Some(number) => {
                let index: u32 =
                    u32::try_from(number).map_err(|_| D::Error::custom("account index is greater than u32::MAX"))?;
                AccountIdentifier::Index(index)
            }
            None => {
                let alias_or_index_str = v
                    .as_str()
                    .ok_or_else(|| D::Error::custom("accountIdentifier is no number or string"))?;
                AccountIdentifier::from(alias_or_index_str)
            }
        })
    }
}

// When the identifier is a string.
impl From<&str> for AccountIdentifier {
    fn from(value: &str) -> Self {
        match u32::from_str(value) {
            Ok(index) => Self::Index(index),
            Err(_) => Self::Alias(value.to_string()),
        }
    }
}

impl From<String> for AccountIdentifier {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for AccountIdentifier {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

// When the identifier is an index.
impl From<u32> for AccountIdentifier {
    fn from(value: u32) -> Self {
        Self::Index(value)
    }
}
