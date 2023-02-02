// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Formatter, Result};

#[cfg(feature = "ledger_nano")]
use iota_client::secret::LedgerNanoStatus;
use iota_client::{
    api::{PreparedTransactionDataDto, SignedTransactionDataDto},
    block::{
        output::{dto::OutputDto, OutputId},
        payload::transaction::TransactionId,
        BlockId,
    },
    NodeInfoWrapper,
};
use serde::Serialize;
#[cfg(feature = "participation")]
use {
    crate::account::operations::participation::{AccountParticipationOverview, ParticipationEventWithNodes},
    iota_client::node_api::participation::types::{ParticipationEventId, ParticipationEventStatus},
    std::collections::HashMap,
};

use crate::{
    account::{
        operations::transaction::high_level::minting::mint_native_token::MintTokenTransactionDto,
        types::{address::AccountAddress, AccountBalanceDto, TransactionDto},
        OutputDataDto,
    },
    message_interface::dtos::{AccountDto, AddressWithUnspentOutputsDto},
    Error,
};

/// The response message.
#[derive(Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Response {
    /// Response for
    /// [`CreateAccount`](crate::message_interface::Message::CreateAccount),
    /// [`GetAccount`](crate::message_interface::Message::GetAccount)
    Account(AccountDto),
    /// Response for [`GetAccountIndexes`](crate::message_interface::Message::GetAccountIndexes)
    AccountIndexes(Vec<u32>),
    /// Response for [`GetAccounts`](crate::message_interface::Message::GetAccounts)
    Accounts(Vec<AccountDto>),
    /// Response for [`Addresses`](crate::message_interface::AccountMethod::Addresses)
    Addresses(Vec<AccountAddress>),
    /// Response for
    /// [`AddressesWithUnspentOutputs`](crate::message_interface::AccountMethod::AddressesWithUnspentOutputs)
    AddressesWithUnspentOutputs(Vec<AddressWithUnspentOutputsDto>),
    /// Response for
    /// [`RetryTransactionUntilIncluded`](crate::message_interface::AccountMethod::RetryTransactionUntilIncluded)
    BlockId(BlockId),
    /// Response for
    /// [`BuildAliasOutput`](crate::message_interface::AccountMethod::BuildAliasOutput)
    /// [`BuildBasicOutput`](crate::message_interface::AccountMethod::BuildBasicOutput)
    /// [`BuildFoundryOutput`](crate::message_interface::AccountMethod::BuildFoundryOutput)
    /// [`BuildNftOutput`](crate::message_interface::AccountMethod::BuildNftOutput)
    /// [`GetFoundryOutput`](crate::message_interface::AccountMethod::GetFoundryOutput)
    /// [`PrepareOutput`](crate::message_interface::AccountMethod::PrepareOutput)
    Output(OutputDto),
    /// Response for
    /// [`MinimumRequiredStorageDeposit`](crate::message_interface::AccountMethod::MinimumRequiredStorageDeposit)
    MinimumRequiredStorageDeposit(String),
    /// Response for
    /// [`GetOutputsWithAdditionalUnlockConditions`](crate::message_interface::AccountMethod::
    /// GetOutputsWithAdditionalUnlockConditions)
    OutputIds(Vec<OutputId>),
    /// Response for [`GetOutput`](crate::message_interface::AccountMethod::GetOutput)
    OutputData(Option<Box<OutputDataDto>>),
    /// Response for
    /// [`Outputs`](crate::message_interface::AccountMethod::Outputs),
    /// [`UnspentOutputs`](crate::message_interface::AccountMethod::UnspentOutputs)
    OutputsData(Vec<OutputDataDto>),
    /// Response for
    /// [`PrepareSendAmount`](crate::message_interface::AccountMethod::PrepareSendAmount),
    /// [`PrepareTransaction`](crate::message_interface::AccountMethod::PrepareTransaction)
    PreparedTransaction(PreparedTransactionDataDto),
    /// Response for
    /// [`GetTransaction`](crate::message_interface::AccountMethod::GetTransaction),
    Transaction(Option<Box<TransactionDto>>),
    /// Response for
    /// [`Transactions`](crate::message_interface::AccountMethod::Transactions),
    /// [`PendingTransactions`](crate::message_interface::AccountMethod::PendingTransactions)
    Transactions(Vec<TransactionDto>),
    /// Response for
    /// [`SignTransaction`](crate::message_interface::AccountMethod::SignTransaction)
    SignedTransactionData(SignedTransactionDataDto),
    /// GenerateAddress response.
    /// Response for [`GenerateAddresses`](crate::message_interface::AccountMethod::GenerateAddresses)
    GeneratedAddress(Vec<AccountAddress>),
    /// Response for
    /// [`GetBalance`](crate::message_interface::AccountMethod::GetBalance),
    /// [`SyncAccount`](crate::message_interface::AccountMethod::SyncAccount)
    Balance(AccountBalanceDto),
    /// Response for
    /// [`GetLedgerNanoStatus`](crate::message_interface::Message::GetLedgerNanoStatus),
    #[cfg(feature = "ledger_nano")]
    LedgerNanoStatus(LedgerNanoStatus),
    /// Response for
    /// [`GetIncomingTransactionData`](crate::message_interface::AccountMethod::GetIncomingTransactionData),
    IncomingTransactionData(Option<Box<(TransactionId, TransactionDto)>>),
    /// Response for
    /// [`IncomingTransactions`](crate::message_interface::AccountMethod::IncomingTransactions),
    IncomingTransactionsData(Vec<(TransactionId, TransactionDto)>),
    /// Response for
    /// [`ConsolidateOutputs`](crate::message_interface::AccountMethod::ConsolidateOutputs)
    /// [`ClaimOutputs`](crate::message_interface::AccountMethod::ClaimOutputs)
    /// [`CreateAliasOutput`](crate::message_interface::AccountMethod::CreateAliasOutput)
    /// [`SendAmount`](crate::message_interface::AccountMethod::SendAmount),
    /// [`MintNfts`](crate::message_interface::AccountMethod::MintNfts),
    /// [`SendAmount`](crate::message_interface::AccountMethod::SendAmount),
    /// [`SendMicroTransaction`](crate::message_interface::AccountMethod::SendMicroTransaction),
    /// [`SendNativeTokens`](crate::message_interface::AccountMethod::SendNativeTokens),
    /// [`SendNft`](crate::message_interface::AccountMethod::SendNft),
    /// [`SendOutputs`](crate::message_interface::AccountMethod::SendOutputs)
    /// [`SubmitAndStoreTransaction`](crate::message_interface::AccountMethod::SubmitAndStoreTransaction)
    /// [`Vote`](crate::message_interface::AccountMethod::Vote)
    /// [`StopParticipating`](crate::message_interface::AccountMethod::StopParticipating)
    /// [`IncreaseVotingPower`](crate::message_interface::AccountMethod::IncreaseVotingPower)
    /// [`DecreaseVotingPower`](crate::message_interface::AccountMethod::DecreaseVotingPower)
    SentTransaction(TransactionDto),
    /// Response for
    /// [`MintNativeToken`](crate::message_interface::AccountMethod::MintNativeToken),
    MintTokenTransaction(MintTokenTransactionDto),
    /// Response for
    /// [`IsStrongholdPasswordAvailable`](crate::message_interface::Message::IsStrongholdPasswordAvailable)
    StrongholdPasswordIsAvailable(bool),
    /// An error occurred.
    Error(Error),
    /// A panic occurred.
    Panic(String),
    /// Response for [`GenerateMnemonic`](crate::message_interface::Message::GenerateMnemonic)
    GeneratedMnemonic(String),
    /// Response for [`GetNodeInfo`](crate::message_interface::Message::GetNodeInfo)
    NodeInfo(NodeInfoWrapper),
    /// Response for
    /// [`GetParticipationEvent`](crate::message_interface::GetParticipationEvent)
    /// [`RegisterParticipationEvent`](crate::message_interface::RegisterParticipationEvent)
    #[cfg(feature = "participation")]
    ParticipationEvent(Option<ParticipationEventWithNodes>),
    /// Response for
    /// [`GetParticipationEventIds`](crate::message_interface::GetParticipationEventIds)
    #[cfg(feature = "participation")]
    ParticipationEventIds(Vec<ParticipationEventId>),
    /// Response for
    /// [`GetParticipationEventStatus`](crate::message_interface::GetParticipationEventStatus)
    #[cfg(feature = "participation")]
    ParticipationEventStatus(ParticipationEventStatus),
    /// Response for
    /// [`GetParticipationEvents`](crate::message_interface::GetParticipationEvents)
    #[cfg(feature = "participation")]
    ParticipationEvents(HashMap<ParticipationEventId, ParticipationEventWithNodes>),
    /// Response for
    /// [`GetVotingPower`](crate::message_interface::AccountMethod::GetVotingPower)
    #[cfg(feature = "participation")]
    VotingPower(String),
    /// Response for
    /// [`GetParticipationOverview`](crate::message_interface::AccountMethod::GetParticipationOverview)
    #[cfg(feature = "participation")]
    AccountParticipationOverview(AccountParticipationOverview),
    /// Response for [`Bech32ToHex`](crate::message_interface::Message::Bech32ToHex)
    HexAddress(String),
    /// Response for [`HexToBech32`](crate::message_interface::Message::HexToBech32)
    /// Response for [`GenerateAddress`](crate::message_interface::Message::GenerateAddress)
    Bech32Address(String),
    /// Response for [`RequestFundsFromFaucet`](crate::message_interface::AccountMethod::RequestFundsFromFaucet)
    Faucet(String),
    /// Response for
    /// [`Backup`](crate::message_interface::Message::Backup),
    /// [`ClearStrongholdPassword`](crate::message_interface::Message::ClearStrongholdPassword),
    /// [`DeregisterParticipationEvent`](crate::message_interface::Message::DeregisterParticipationEvent),
    /// [`RestoreBackup`](crate::message_interface::Message::RestoreBackup),
    /// [`VerifyMnemonic`](crate::message_interface::Message::VerifyMnemonic),
    /// [`SetClientOptions`](crate::message_interface::Message::SetClientOptions),
    /// [`SetStrongholdPassword`](crate::message_interface::Message::SetStrongholdPassword),
    /// [`SetStrongholdPasswordClearInterval`](crate::message_interface::Message::
    /// SetStrongholdPasswordClearInterval),
    /// [`StoreMnemonic`](crate::message_interface::Message::StoreMnemonic),
    /// [`StartBackgroundSync`](crate::message_interface::Message::StartBackgroundSync),
    /// [`StopBackgroundSync`](crate::message_interface::Message::StopBackgroundSync),
    /// [`EmitTestEvent`](crate::message_interface::Message::EmitTestEvent),
    Ok(()),
}

// Custom Debug implementation to not log secrets
impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Account(account) => write!(f, "Account({account:?})"),
            Self::AccountIndexes(account_indexes) => write!(f, "AccountIndexes({account_indexes:?})"),
            Self::Accounts(accounts) => write!(f, "Accounts({accounts:?})"),
            Self::Addresses(addresses) => write!(f, "Addresses({addresses:?})"),
            Self::AddressesWithUnspentOutputs(addresses) => {
                write!(f, "AddressesWithUnspentOutputs({addresses:?})")
            }
            Self::BlockId(block_id) => write!(f, "BlockId({block_id:?})"),
            Self::Output(output) => write!(f, "Output({output:?})"),
            Self::MinimumRequiredStorageDeposit(amount) => write!(f, "MinimumRequiredStorageDeposit({amount:?})"),
            Self::OutputIds(output_ids) => write!(f, "OutputIds({output_ids:?})"),
            Self::OutputData(output) => write!(f, "OutputData({output:?})"),
            Self::OutputsData(outputs) => write!(f, "OutputsData{outputs:?}"),
            Self::PreparedTransaction(transaction_data) => {
                write!(f, "PreparedTransaction({transaction_data:?})")
            }
            Self::Transaction(transaction) => write!(f, "Transaction({transaction:?})"),
            Self::Transactions(transactions) => write!(f, "Transactions({transactions:?})"),
            Self::SignedTransactionData(signed_transaction_data) => {
                write!(f, "SignedTransactionData({signed_transaction_data:?})")
            }
            Self::GeneratedAddress(addresses) => write!(f, "GeneratedAddress({addresses:?})"),
            Self::Balance(balance) => write!(f, "Balance({balance:?})"),
            Self::IncomingTransactionData(transaction_data) => {
                write!(f, "IncomingTransactionData({transaction_data:?})")
            }
            Self::IncomingTransactionsData(transactions_data) => {
                write!(f, "IncomingTransactionsData({transactions_data:?})")
            }
            Self::SentTransaction(transaction) => write!(f, "SentTransaction({transaction:?})"),
            Self::MintTokenTransaction(mint_transaction) => {
                write!(f, "MintTokenTransaction({mint_transaction:?})")
            }
            Self::StrongholdPasswordIsAvailable(is_available) => {
                write!(f, "StrongholdPasswordIsAvailable({is_available:?})")
            }
            Self::Error(error) => write!(f, "Error({error:?})"),
            Self::Panic(panic_msg) => write!(f, "Panic({panic_msg:?})"),
            Self::GeneratedMnemonic(_) => write!(f, "GeneratedMnemonic(<omitted>)"),
            #[cfg(feature = "ledger_nano")]
            Self::LedgerNanoStatus(ledger_nano_status) => write!(f, "LedgerNanoStatus({ledger_nano_status:?})"),
            Self::NodeInfo(info) => write!(f, "NodeInfo({info:?})"),
            Self::HexAddress(hex_address) => write!(f, "Hex encoded address({hex_address:?})"),
            Self::Bech32Address(bech32_address) => write!(f, "Bech32 encoded address({bech32_address:?})"),
            Self::Ok(()) => write!(f, "Ok(())"),
            #[cfg(feature = "participation")]
            Self::ParticipationEvent(event) => write!(f, "ParticipationEvent({event:?})"),
            #[cfg(feature = "participation")]
            Self::ParticipationEventStatus(event_status) => write!(f, "ParticipationEventStatus({event_status:?})"),
            #[cfg(feature = "participation")]
            Self::ParticipationEvents(events) => write!(f, "ParticipationEvents({events:?})"),
            #[cfg(feature = "participation")]
            Self::ParticipationEventIds(event_ids) => write!(f, "ParticipationEventIds({event_ids:?})"),
            #[cfg(feature = "participation")]
            Self::VotingPower(amount) => write!(f, "VotingPower({amount:?})"),
            #[cfg(feature = "participation")]
            Self::AccountParticipationOverview(overview) => {
                write!(f, "AccountParticipationOverview({overview:?})")
            }
            Self::Faucet(response) => write!(f, "Faucet({response:?})"),
        }
    }
}
