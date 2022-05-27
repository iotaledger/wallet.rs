// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::{PreparedTransactionDataDto, SignedTransactionDataDto},
    bee_block::{
        output::{
            dto::{NativeTokenDto, OutputDto},
            feature::dto::FeatureDto,
            unlock_condition::dto::UnlockConditionDto,
            OutputId,
        },
        payload::transaction::TransactionId,
    },
};
use serde::Deserialize;

use crate::{
    account::operations::{
        address_generation::AddressGenerationOptions, output_collection::OutputsToCollect, syncing::SyncOptions,
        transaction::TransactionOptions,
    },
    message_interface::dtos::{AddressWithAmountDto, AddressWithMicroAmountDto},
    AddressAndNftId, AddressNativeTokens, NativeTokenOptions, NftOptions,
};

/// Each public account method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum AccountMethod {
    /// Build a basic output.
    /// Expected response: [`BuiltOutput`](crate::message_interface::Response::BuiltOutput)
    BuildBasicOutput {
        // If not provided, minimum storage deposit will be used
        amount: Option<String>,
        #[serde(rename = "nativeTokens")]
        native_tokens: Option<Vec<NativeTokenDto>>,
        #[serde(rename = "unlockConditions")]
        unlock_conditions: Vec<UnlockConditionDto>,
        features: Option<Vec<FeatureDto>>,
    },
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
    /// Get the [`Transaction`](crate::account::types::Transaction) of a transaction stored in the account
    /// Expected response: [`Transaction`](crate::message_interface::Response::Transaction)
    GetTransaction {
        #[serde(rename = "transactionId")]
        transaction_id: TransactionId,
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
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    MintNativeToken {
        #[serde(rename = "nativeTokenOptions")]
        native_token_options: NativeTokenOptions,
        options: Option<TransactionOptions>,
    },
    /// Mint nft.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    MintNfts {
        #[serde(rename = "nftOptions")]
        nfts_options: Vec<NftOptions>,
        options: Option<TransactionOptions>,
    },
    /// Get account balance information.
    /// Expected response: [`Balance`](crate::message_interface::Response::Balance)
    GetBalance,
    /// Prepare transaction.
    /// Expected response: [`PreparedTransactionData`](crate::message_interface::Response::PreparedTransactionData)
    PrepareTransaction {
        outputs: Vec<OutputDto>,
        options: Option<TransactionOptions>,
    },
    /// Prepare mint nft.
    /// Expected response: [`PreparedTransactionData`](crate::message_interface::Response::PreparedTransactionData)
    PrepareMintNfts {
        #[serde(rename = "nftOptions")]
        nfts_options: Vec<NftOptions>,
        options: Option<TransactionOptions>,
    },
    /// Prepare send amount.
    /// Expected response: [`PreparedTransactionData`](crate::message_interface::Response::PreparedTransactionData)
    PrepareSendAmount {
        #[serde(rename = "addressWithAmount")]
        addresses_with_amount: Vec<AddressWithAmountDto>,
        options: Option<TransactionOptions>,
    },
    /// Prepare send amount below minimum storage deposit.
    /// Expected response: [`PreparedTransactionData`](crate::message_interface::Response::PreparedTransactionData)
    PrepareSendMicroTransaction {
        #[serde(rename = "addressWithMicroAmount")]
        addresses_with_micro_amount: Vec<AddressWithMicroAmountDto>,
        options: Option<TransactionOptions>,
    },
    /// Prepare send native tokens.
    /// Expected response: [`PreparedTransactionData`](crate::message_interface::Response::PreparedTransactionData)
    PrepareSendNativeTokens {
        #[serde(rename = "addressNativeTokens")]
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransactionOptions>,
    },
    /// Prepare send nft.
    /// Expected response: [`PreparedTransactionData`](crate::message_interface::Response::PreparedTransactionData)
    PrepareSendNft {
        #[serde(rename = "addressAndNftId")]
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptions>,
    },
    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    /// Expected response: [`Balance`](crate::message_interface::Response::Balance)
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>,
    },
    /// Send amount.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendAmount {
        #[serde(rename = "addressWithAmount")]
        addresses_with_amount: Vec<AddressWithAmountDto>,
        options: Option<TransactionOptions>,
    },
    /// Send amount below minimum storage deposit.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendMicroTransaction {
        #[serde(rename = "addressWithMicroAmount")]
        addresses_with_micro_amount: Vec<AddressWithMicroAmountDto>,
        options: Option<TransactionOptions>,
    },
    /// Send native tokens.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendNativeTokens {
        #[serde(rename = "addressNativeTokens")]
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransactionOptions>,
    },
    /// Send nft.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendNft {
        #[serde(rename = "addressAndNftId")]
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptions>,
    },
    /// Set the alias of the account.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    SetAlias { alias: String },
    /// Send funds.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendTransaction {
        outputs: Vec<OutputDto>,
        options: Option<TransactionOptions>,
    },
    /// Sign a prepared transaction.
    /// Expected response: [`TransactionPayload`](crate::message_interface::Response::TransactionPayload)
    SignTransactionEssence {
        #[serde(rename = "preparedTransactionData")]
        prepared_transaction_data: PreparedTransactionDataDto,
    },
    /// Validate the transaction, submit it to a node and store it in the account.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SubmitAndStoreTransaction {
        #[serde(rename = "signedTransactionData")]
        signed_transaction_data: SignedTransactionDataDto,
    },
    /// Try to collect outputs.
    /// Expected response: [`SentTransactions`](crate::message_interface::Response::SentTransactions)
    TryCollectOutputs {
        #[serde(rename = "outputsToCollect")]
        outputs_to_collect: OutputsToCollect,
    },
    /// Collect outputs.
    /// Expected response: [`SentTransactions`](crate::message_interface::Response::SentTransactions)
    CollectOutputs {
        #[serde(rename = "outputsToCollect")]
        output_ids_to_collect: Vec<OutputId>,
    },
}
