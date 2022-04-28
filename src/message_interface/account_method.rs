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
    /// Expected response: [`GeneratedAddress`](crate::message_interface::ResponseType::GeneratedAddress)
    GenerateAddresses {
        amount: u32,
        options: Option<AddressGenerationOptions>,
    },
    /// Get outputs with additional unlock conditions
    /// Expected response: [`OutputIds`](crate::message_interface::ResponseType::OutputIds)
    GetOutputsWithAdditionalUnlockConditions {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// Expected response: [`Addresses`](crate::message_interface::ResponseType::Addresses)
    /// List addresses.
    ListAddresses,
    /// Returns only addresses of the account with unspent outputs
    /// Expected response:
    /// [`AddressesWithUnspentOutputs`](crate::message_interface::ResponseType::AddressesWithUnspentOutputs)
    ListAddressesWithUnspentOutputs,
    /// Returns all outputs of the account
    /// Expected response: [`Outputs`](crate::message_interface::ResponseType::Outputs)
    ListOutputs,
    /// Returns all unspent outputs of the account
    /// Expected response: [`Outputs`](crate::message_interface::ResponseType::Outputs)
    ListUnspentOutputs,
    /// Returns all transaction of the account
    /// Expected response: [`Transactions`](crate::message_interface::ResponseType::Transactions)
    ListTransactions,
    /// Returns all pending transaction of the account
    /// Expected response: [`Transactions`](crate::message_interface::ResponseType::Transactions)
    ListPendingTransactions,
    /// Mint native token.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    MintNativeToken {
        #[serde(rename = "nativeTokenOptions")]
        native_token_options: NativeTokenOptions,
        options: Option<TransferOptions>,
    },
    /// Mint nft.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    MintNfts {
        #[serde(rename = "nftOptions")]
        nfts_options: Vec<NftOptions>,
        options: Option<TransferOptions>,
    },
    /// Get account balance information.
    /// Expected response: [`Balance`](crate::message_interface::ResponseType::Balance)
    GetBalance,
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    /// Expected response: [`Balance`](crate::message_interface::ResponseType::Balance)
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>,
    },
    /// Send amount.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    SendAmount {
        #[serde(rename = "addressWithAmount")]
        addresses_with_amount: Vec<AddressWithAmount>,
        options: Option<TransferOptions>,
    },
    /// Send amount below minimum storage deposit.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    SendMicroTransaction {
        #[serde(rename = "addressWithMicroAmount")]
        addresses_with_micro_amount: Vec<AddressWithMicroAmount>,
        options: Option<TransferOptions>,
    },
    /// Send native tokens.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    SendNativeTokens {
        #[serde(rename = "addressNativeTokens")]
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransferOptions>,
    },
    /// Send nft.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    SendNft {
        #[serde(rename = "addressAndNftId")]
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransferOptions>,
    },
    /// Send funds.
    /// Expected response: [`SentTransfer`](crate::message_interface::ResponseType::SentTransfer)
    SendTransfer {
        outputs: Vec<Output>,
        options: Option<TransferOptions>,
    },
    /// Try to collect outputs.
    /// Expected response: [`SentTransfers`](crate::message_interface::ResponseType::SentTransfers)
    TryCollectOutputs {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// Collect outputs.
    /// Expected response: [`SentTransfers`](crate::message_interface::ResponseType::SentTransfers)
    CollectOutputs {
        #[serde(rename = "outputsToCollect")]
        output_ids_to_collect: Vec<OutputId>,
    },
}
