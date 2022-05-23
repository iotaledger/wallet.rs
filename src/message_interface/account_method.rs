// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_message::output::{Output, OutputId};
use serde::Deserialize;

use crate::{
    account::operations::{
        address_generation::AddressGenerationOptions, output_collection::OutputsToCollect, syncing::SyncOptions,
        transfer::TransferOptions,
    },
    message_interface::dtos::{AddressWithAmountDto, AddressWithMicroAmountDto},
    AddressAndNftId, AddressNativeTokens, NativeTokenOptions, NftOptions,
};

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Generate a new unused address.
    /// Expected response: [`GeneratedAddress`](crate::message_interface::Response::GeneratedAddress)
    GenerateAddresses {
        amount: u32,
        options: Option<AddressGenerationOptions>,
    },
    /// Get the [`OutputData`](crate::account::types::OutputData) of an output stored in the account
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    GetOutput {
        #[serde(rename = "outputId")]
        output_id: OutputId,
    },
    /// Get outputs with additional unlock conditions
    /// Expected response: [`OutputIds`](crate::message_interface::Response::OutputIds)
    GetOutputsWithAdditionalUnlockConditions {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// Expected response: [`Addresses`](crate::message_interface::Response::Addresses)
    /// List addresses.
    ListAddresses,
    /// Returns only addresses of the account with unspent outputs
    /// Expected response:
    /// [`AddressesWithUnspentOutputs`](crate::message_interface::Response::AddressesWithUnspentOutputs)
    ListAddressesWithUnspentOutputs,
    /// Returns all outputs of the account
    /// Expected response: [`Outputs`](crate::message_interface::Response::Outputs)
    ListOutputs,
    /// Returns all unspent outputs of the account
    /// Expected response: [`Outputs`](crate::message_interface::Response::Outputs)
    ListUnspentOutputs,
    /// Returns all transaction of the account
    /// Expected response: [`Transactions`](crate::message_interface::Response::Transactions)
    ListTransactions,
    /// Returns all pending transaction of the account
    /// Expected response: [`Transactions`](crate::message_interface::Response::Transactions)
    ListPendingTransactions,
    /// Mint native token.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    MintNativeToken {
        #[serde(rename = "nativeTokenOptions")]
        native_token_options: NativeTokenOptions,
        options: Option<TransferOptions>,
    },
    /// Mint nft.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    MintNfts {
        #[serde(rename = "nftOptions")]
        nfts_options: Vec<NftOptions>,
        options: Option<TransferOptions>,
    },
    /// Get account balance information.
    /// Expected response: [`Balance`](crate::message_interface::Response::Balance)
    GetBalance,
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    /// Expected response: [`Balance`](crate::message_interface::Response::Balance)
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>,
    },
    /// Send amount.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    SendAmount {
        #[serde(rename = "addressWithAmount")]
        addresses_with_amount: Vec<AddressWithAmountDto>,
        options: Option<TransferOptions>,
    },
    /// Send amount below minimum storage deposit.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    SendMicroTransaction {
        #[serde(rename = "addressWithMicroAmount")]
        addresses_with_micro_amount: Vec<AddressWithMicroAmountDto>,
        options: Option<TransferOptions>,
    },
    /// Send native tokens.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    SendNativeTokens {
        #[serde(rename = "addressNativeTokens")]
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransferOptions>,
    },
    /// Send nft.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    SendNft {
        #[serde(rename = "addressAndNftId")]
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransferOptions>,
    },
    /// Set the alias of the account.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    SetAlias { alias: String },
    /// Send funds.
    /// Expected response: [`SentTransfer`](crate::message_interface::Response::SentTransfer)
    SendTransfer {
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
    },
    /// Try to collect outputs.
    /// Expected response: [`SentTransfers`](crate::message_interface::Response::SentTransfers)
    TryCollectOutputs {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// Collect outputs.
    /// Expected response: [`SentTransfers`](crate::message_interface::Response::SentTransfers)
    CollectOutputs {
        #[serde(rename = "outputsToCollect")]
        output_ids_to_collect: Vec<OutputId>,
    },
}
