// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api::{PreparedTransactionDataDto, SignedTransactionDataDto},
    block::{
        dto::U256Dto,
        output::{
            dto::{AliasIdDto, NativeTokenDto, NftIdDto, OutputDto, TokenIdDto, TokenSchemeDto},
            feature::dto::FeatureDto,
            unlock_condition::dto::UnlockConditionDto,
            FoundryId, OutputId,
        },
        payload::transaction::TransactionId,
    },
};
#[cfg(feature = "participation")]
use iota_client::{
    api_types::plugins::participation::types::{ParticipationEventId, ParticipationEventType},
    node_manager::node::Node,
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        handle::FilterOptions,
        operations::{
            address_generation::AddressGenerationOptions,
            output_claiming::OutputsToClaim,
            syncing::SyncOptions,
            transaction::{
                high_level::{
                    create_alias::AliasOutputOptionsDto,
                    minting::{
                        increase_native_token_supply::IncreaseNativeTokenSupplyOptionsDto,
                        mint_native_token::NativeTokenOptionsDto, mint_nfts::NftOptionsDto,
                    },
                },
                prepare_output::OutputOptionsDto,
                TransactionOptionsDto,
            },
        },
    },
    message_interface::dtos::{AddressWithAmountDto, AddressWithMicroAmountDto},
    AddressAndNftId, AddressNativeTokens,
};

/// Each public account method.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "name", content = "data", rename_all = "camelCase")]
pub enum AccountMethod {
    /// Build an AliasOutput.
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    #[allow(missing_docs)]
    BuildAliasOutput {
        // If not provided, minimum storage deposit will be used
        amount: Option<String>,
        #[serde(rename = "nativeTokens")]
        native_tokens: Option<Vec<NativeTokenDto>>,
        #[serde(rename = "aliasId")]
        alias_id: AliasIdDto,
        #[serde(rename = "stateIndex")]
        state_index: Option<u32>,
        #[serde(rename = "stateMetadata")]
        state_metadata: Option<Vec<u8>>,
        #[serde(rename = "foundryCounter")]
        foundry_counter: Option<u32>,
        #[serde(rename = "unlockConditions")]
        unlock_conditions: Vec<UnlockConditionDto>,
        features: Option<Vec<FeatureDto>>,
        #[serde(rename = "immutableFeatures")]
        immutable_features: Option<Vec<FeatureDto>>,
    },
    /// Build a BasicOutput.
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    #[allow(missing_docs)]
    BuildBasicOutput {
        // If not provided, minimum storage deposit will be used
        amount: Option<String>,
        #[serde(rename = "nativeTokens")]
        native_tokens: Option<Vec<NativeTokenDto>>,
        #[serde(rename = "unlockConditions")]
        unlock_conditions: Vec<UnlockConditionDto>,
        features: Option<Vec<FeatureDto>>,
    },
    /// Build a FoundryOutput.
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    #[allow(missing_docs)]
    BuildFoundryOutput {
        // If not provided, minimum storage deposit will be used
        amount: Option<String>,
        #[serde(rename = "nativeTokens")]
        native_tokens: Option<Vec<NativeTokenDto>>,
        #[serde(rename = "serialNumber")]
        serial_number: u32,
        #[serde(rename = "tokenScheme")]
        token_scheme: TokenSchemeDto,
        #[serde(rename = "unlockConditions")]
        unlock_conditions: Vec<UnlockConditionDto>,
        features: Option<Vec<FeatureDto>>,
        #[serde(rename = "immutableFeatures")]
        immutable_features: Option<Vec<FeatureDto>>,
    },
    /// Build an NftOutput.
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    #[allow(missing_docs)]
    BuildNftOutput {
        // If not provided, minimum storage deposit will be used
        amount: Option<String>,
        #[serde(rename = "nativeTokens")]
        native_tokens: Option<Vec<NativeTokenDto>>,
        #[serde(rename = "nftId")]
        nft_id: NftIdDto,
        #[serde(rename = "unlockConditions")]
        unlock_conditions: Vec<UnlockConditionDto>,
        features: Option<Vec<FeatureDto>>,
        #[serde(rename = "immutableFeatures")]
        immutable_features: Option<Vec<FeatureDto>>,
    },
    /// Burn native tokens. This doesn't require the foundry output which minted them, but will not increase
    /// the foundries `melted_tokens` field, which makes it impossible to destroy the foundry output. Therefore it's
    /// recommended to use melting, if the foundry output is available.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    BurnNativeToken {
        /// Native token id
        #[serde(rename = "tokenId")]
        token_id: TokenIdDto,
        /// To be burned amount
        #[serde(rename = "burnAmount")]
        burn_amount: U256Dto,
        options: Option<TransactionOptionsDto>,
    },
    /// Burn an nft output. Outputs controlled by it will be swept before if they don't have a storage
    /// deposit return, timelock or expiration unlock condition. This should be preferred over burning, because after
    /// burning, the foundry can never be destroyed anymore.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    BurnNft {
        #[serde(rename = "nftId")]
        nft_id: NftIdDto,
        options: Option<TransactionOptionsDto>,
    },
    /// Consolidate outputs.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    ConsolidateOutputs {
        force: bool,
        #[serde(rename = "outputConsolidationThreshold")]
        output_consolidation_threshold: Option<usize>,
    },
    /// Create an alias output.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    CreateAliasOutput {
        #[serde(rename = "aliasOutputOptions")]
        alias_output_options: Option<AliasOutputOptionsDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Destroy an alias output. Outputs controlled by it will be swept before if they don't have a
    /// storage deposit return, timelock or expiration unlock condition. The amount and possible native tokens will be
    /// sent to the governor address.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    DestroyAlias {
        #[serde(rename = "aliasId")]
        alias_id: AliasIdDto,
        options: Option<TransactionOptionsDto>,
    },
    /// Function to destroy a foundry output with a circulating supply of 0.
    /// Native tokens in the foundry (minted by other foundries) will be transacted to the controlling alias
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    DestroyFoundry {
        #[serde(rename = "foundryId")]
        foundry_id: FoundryId,
        options: Option<TransactionOptionsDto>,
    },
    /// Generate new unused addresses.
    /// Expected response: [`GeneratedAddress`](crate::message_interface::Response::GeneratedAddress)
    GenerateAddresses {
        amount: u32,
        options: Option<AddressGenerationOptions>,
    },
    /// Get the [`OutputData`](crate::account::types::OutputData) of an output stored in the account
    /// Expected response: [`OutputData`](crate::message_interface::Response::OutputData)
    GetOutput {
        #[serde(rename = "outputId")]
        output_id: OutputId,
    },
    /// Get the [`Output`](crate::iota_client::block::output::Output) that minted a native token by its TokenId
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    GetFoundryOutput {
        #[serde(rename = "tokenId")]
        token_id: TokenIdDto,
    },
    /// Get outputs with additional unlock conditions
    /// Expected response: [`OutputIds`](crate::message_interface::Response::OutputIds)
    GetOutputsWithAdditionalUnlockConditions {
        #[serde(rename = "outputsToClaim")]
        outputs_to_claim: OutputsToClaim,
    },
    /// Get the [`Transaction`](crate::account::types::Transaction) of a transaction stored in the account
    /// Expected response: [`Transaction`](crate::message_interface::Response::Transaction)
    GetTransaction {
        #[serde(rename = "transactionId")]
        transaction_id: TransactionId,
    },
    /// Get the transaction with inputs of an incoming transaction stored in the account
    /// List might not be complete, if the node pruned the data already
    /// Expected response: [`IncomingTransactionData`](crate::message_interface::Response::IncomingTransactionData)
    GetIncomingTransactionData {
        #[serde(rename = "transactionId")]
        transaction_id: TransactionId,
    },
    /// Expected response: [`Addresses`](crate::message_interface::Response::Addresses)
    /// List addresses.
    Addresses,
    /// Returns only addresses of the account with unspent outputs
    /// Expected response:
    /// [`AddressesWithUnspentOutputs`](crate::message_interface::Response::AddressesWithUnspentOutputs)
    AddressesWithUnspentOutputs,
    /// Returns all outputs of the account
    /// Expected response: [`OutputsData`](crate::message_interface::Response::OutputsData)
    Outputs {
        #[serde(rename = "filterOptions")]
        filter_options: Option<FilterOptions>,
    },
    /// Returns all unspent outputs of the account
    /// Expected response: [`OutputsData`](crate::message_interface::Response::OutputsData)
    UnspentOutputs {
        #[serde(rename = "filterOptions")]
        filter_options: Option<FilterOptions>,
    },
    /// Returns all incoming transactions of the account
    /// Expected response: [`IncomingTransactionsData`](crate::message_interface::Response::IncomingTransactionsData)
    IncomingTransactions,
    /// Returns all transaction of the account
    /// Expected response: [`Transactions`](crate::message_interface::Response::Transactions)
    Transactions,
    /// Returns all pending transactions of the account
    /// Expected response: [`Transactions`](crate::message_interface::Response::Transactions)
    PendingTransactions,
    /// Melt native tokens. This happens with the foundry output which minted them, by increasing it's
    /// `melted_tokens` field.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    DecreaseNativeTokenSupply {
        /// Native token id
        #[serde(rename = "tokenId")]
        token_id: TokenIdDto,
        /// To be melted amount
        #[serde(rename = "meltAmount")]
        melt_amount: U256Dto,
        options: Option<TransactionOptionsDto>,
    },
    /// Calculate the minimum required storage deposit for an output.
    /// Expected response:
    /// [`MinimumRequiredStorageDeposit`](crate::message_interface::Response::MinimumRequiredStorageDeposit)
    MinimumRequiredStorageDeposit { output: OutputDto },
    /// Mint more native token.
    /// Expected response: [`MintTokenTransaction`](crate::message_interface::Response::MintTokenTransaction)
    IncreaseNativeTokenSupply {
        /// Native token id
        #[serde(rename = "tokenId")]
        token_id: TokenIdDto,
        /// To be minted amount
        #[serde(rename = "mintAmount")]
        mint_amount: U256Dto,
        #[serde(rename = "increaseNativeTokenSupplyOptions")]
        increase_native_token_supply_options: Option<IncreaseNativeTokenSupplyOptionsDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Mint native token.
    /// Expected response: [`MintTokenTransaction`](crate::message_interface::Response::MintTokenTransaction)
    MintNativeToken {
        #[serde(rename = "nativeTokenOptions")]
        native_token_options: NativeTokenOptionsDto,
        options: Option<TransactionOptionsDto>,
    },
    /// Mint nft.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    MintNfts {
        #[serde(rename = "nftsOptions")]
        nfts_options: Vec<NftOptionsDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Get account balance information.
    /// Expected response: [`Balance`](crate::message_interface::Response::Balance)
    GetBalance,
    /// Prepare an output.
    /// Expected response: [`Output`](crate::message_interface::Response::Output)
    PrepareOutput {
        options: OutputOptionsDto,
        transaction_options: Option<TransactionOptionsDto>,
    },
    /// Prepare transaction.
    /// Expected response: [`PreparedTransaction`](crate::message_interface::Response::PreparedTransaction)
    PrepareTransaction {
        outputs: Vec<OutputDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Prepare send amount.
    /// Expected response: [`PreparedTransaction`](crate::message_interface::Response::PreparedTransaction)
    PrepareSendAmount {
        #[serde(rename = "addressesWithAmount")]
        addresses_with_amount: Vec<AddressWithAmountDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Retries (promotes or reattaches) a transaction sent from the account for a provided transaction id until it's
    /// included (referenced by a milestone). Returns the included block id.
    /// Expected response: [`BlockId`](crate::message_interface::Response::BlockId)
    RetryTransactionUntilIncluded {
        /// Transaction id
        #[serde(rename = "transactionId")]
        transaction_id: TransactionId,
        /// Interval
        interval: Option<u64>,
        /// Maximum attempts
        #[serde(rename = "maxAttempts")]
        max_attempts: Option<u64>,
    },
    /// Sync the account by fetching new information from the nodes. Will also retry pending transactions
    /// if necessary.
    /// Expected response: [`Balance`](crate::message_interface::Response::Balance)
    SyncAccount {
        /// Sync options
        options: Option<SyncOptions>,
    },
    /// Send amount.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendAmount {
        #[serde(rename = "addressesWithAmount")]
        addresses_with_amount: Vec<AddressWithAmountDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Send amount below minimum storage deposit.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendMicroTransaction {
        #[serde(rename = "addressesWithMicroAmount")]
        addresses_with_micro_amount: Vec<AddressWithMicroAmountDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Send native tokens.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendNativeTokens {
        #[serde(rename = "addressesNativeTokens")]
        addresses_native_tokens: Vec<AddressNativeTokens>,
        options: Option<TransactionOptionsDto>,
    },
    /// Send nft.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendNft {
        #[serde(rename = "addressesAndNftIds")]
        addresses_nft_ids: Vec<AddressAndNftId>,
        options: Option<TransactionOptionsDto>,
    },
    /// Set the alias of the account.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    SetAlias { alias: String },
    /// Send outputs in a transaction.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    SendOutputs {
        outputs: Vec<OutputDto>,
        options: Option<TransactionOptionsDto>,
    },
    /// Sign a prepared transaction.
    /// Expected response: [`SignedTransactionData`](crate::message_interface::Response::SignedTransactionData)
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
    /// Claim outputs.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    ClaimOutputs {
        #[serde(rename = "outputIdsToClaim")]
        output_ids_to_claim: Vec<OutputId>,
    },
    /// Vote for a participation event.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    Vote {
        #[serde(rename = "eventId")]
        event_id: Option<ParticipationEventId>,
        answers: Option<Vec<u8>>,
    },
    /// Stop participating for an event.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    StopParticipating {
        #[serde(rename = "eventId")]
        event_id: ParticipationEventId,
    },
    /// Get the account's total voting power (voting or NOT voting).
    /// Expected response: [`VotingPower`](crate::message_interface::Response::VotingPower)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    GetVotingPower,
    /// Calculates a participation overview for an account.
    /// Expected response:
    /// [`AccountParticipationOverview`](crate::message_interface::Response::AccountParticipationOverview)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    GetParticipationOverview,
    /// Designates a given amount of tokens towards an account's "voting power" by creating a
    /// special output, which is really a basic one with some metadata.
    /// This will stop voting in most cases (if there is a remainder output), but the voting data isn't lost and
    /// calling `Vote` without parameters will revote. Expected response:
    /// [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    IncreaseVotingPower { amount: String },
    /// Reduces an account's "voting power" by a given amount.
    /// This will stop voting, but the voting data isn't lost and calling `Vote` without parameters will revote.
    /// Expected response: [`SentTransaction`](crate::message_interface::Response::SentTransaction)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    DecreaseVotingPower { amount: String },
    /// Stores participation information locally and returns the event.
    ///
    /// This will NOT store the node url and auth inside the client options.
    /// Expected response: [`ParticipationEvent`](crate::message_interface::Response::ParticipationEvent)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    RegisterParticipationEvent {
        #[serde(rename = "eventId")]
        event_id: ParticipationEventId,
        nodes: Vec<Node>,
    },
    /// Removes a previously registered participation event from local storage.
    /// Expected response: [`Ok`](crate::message_interface::Response::Ok)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    DeregisterParticipationEvent {
        #[serde(rename = "eventId")]
        event_id: ParticipationEventId,
    },
    /// Expected response: [`ParticipationEvent`](crate::message_interface::Response::ParticipationEvent)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    GetParticipationEvent {
        #[serde(rename = "eventId")]
        event_id: ParticipationEventId,
    },
    /// Expected response: [`ParticipationEventIds`](crate::message_interface::Response::ParticipationEventIds)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    GetParticipationEventIds {
        #[serde(rename = "eventType")]
        event_type: Option<ParticipationEventType>,
    },
    /// Expected response: [`ParticipationEventStatus`](crate::message_interface::Response::ParticipationEventStatus)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    GetParticipationEventStatus {
        #[serde(rename = "eventId")]
        event_id: ParticipationEventId,
    },
    /// Expected response: [`ParticipationEvents`](crate::message_interface::Response::ParticipationEvents)
    #[cfg(feature = "participation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "participation")))]
    GetParticipationEvents,
    /// Expected response: [`Faucet`](crate::message_interface::Response::Faucet)
    RequestFundsFromFaucet { url: String, address: String },
}
