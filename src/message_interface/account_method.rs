// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_message::output::{Output, OutputId};
use serde::Deserialize;

use crate::{
    account::operations::{
        address_generation::AddressGenerationOptions, output_collection::OutputsToCollect, syncing::SyncOptions,
        transfer::TransferOptions,
    },
    AddressAndNftId, AddressNativeTokens, AddressWithAmount, AddressWithMicroAmount, NativeTokenOptions, NftOptions,
};

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Generate a new unused address.
    GenerateAddresses {
        amount: u32,
        options: Option<AddressGenerationOptions>,
    },
    /// Get outputs with additional unlock conditions
    GetOutputsWithAdditionalUnlockConditions {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// List addresses.
    ListAddresses,
    /// Returns only addresses of the account with unspent outputs
    ListAddressesWithUnspentOutputs,
    /// Returns all outputs of the account
    ListOutputs,
    /// Returns all unspent outputs of the account
    ListUnspentOutputs,
    /// Returns all transaction of the account
    ListTransactions,
    /// Returns all pending transaction of the account
    ListPendingTransactions,
    /// Mint native token.
    MintNativeToken {
        #[serde(rename = "nativeTokenOptions")]
        native_token_options: NativeTokenOptions,
        options: Option<TransferOptions>,
    },
    /// Mint nft.
    MintNfts {
        #[serde(rename = "nftOptions")]
        nfts_options: Vec<NftOptions>,
        options: Option<TransferOptions>,
    },
    /// Get account balance information.
    GetBalance,
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>,
    },
    /// Send amount.
    SendAmount {
        #[serde(rename = "addressWithAmount")]
        addresses_with_amount: Vec<AddressWithAmount>,
        options: Option<TransferOptions>,
    },
    // /// Send amount below minimum storage deposit.
    SendMicroTransaction {
        #[serde(rename = "addressWithMicroAmount")]
        addresses_with_micro_amount: Vec<AddressWithMicroAmount>,
        options: Option<TransferOptions>,
    },
    /// Send native tokens.
    SendNativeTokens {
        #[serde(rename = "addressNativeTokens")]
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransferOptions>,
    },
    /// Send nft.
    SendNft {
        #[serde(rename = "addressAndNftId")]
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransferOptions>,
    },
    /// Send funds.
    SendTransfer {
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
    },
    /// Try to collect outputs.
    TryCollectOutputs {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// Collect outputs.
    CollectOutputs {
        #[serde(rename = "outputsToCollect")]
        output_ids_to_collect: Vec<OutputId>,
    },
}
