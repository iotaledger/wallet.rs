// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The module with the AccountBuilder.
pub(crate) mod builder;
/// Constants used for the account and account operations.
pub(crate) mod constants;
/// A thread guard over an account, all account methods are called from here.
pub(crate) mod handle;
/// The account operations like address generation, syncing and creating transactions.
pub(crate) mod operations;
/// Types used in an account and returned from methods.
pub mod types;
/// Methods to update the account state.
pub(crate) mod update;

use std::collections::{HashMap, HashSet};

use getset::{Getters, Setters};
use iota_client::{
    api_types::response::OutputWithMetadataResponse,
    block::{
        output::{FoundryId, FoundryOutput, OutputId},
        payload::{transaction::TransactionId, TransactionPayload},
    },
};
use serde::{Deserialize, Serialize};

use self::types::{
    address::{AccountAddress, AddressWithUnspentOutputs},
    AccountBalance, OutputData,
};
pub use self::{
    handle::AccountHandle,
    operations::{
        address_generation::AddressGenerationOptions,
        output_claiming::OutputsToClaim,
        syncing::SyncOptions,
        transaction::{
            prepare_output::{Assets, Features, OutputOptions, StorageDeposit, Unlocks},
            RemainderValueStrategy, TransactionOptions,
        },
    },
    types::OutputDataDto,
};

/// An Account.
#[derive(Clone, Debug, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Account {
    /// The account index
    index: u32,
    /// The coin type
    #[serde(rename = "coinType")]
    coin_type: u32,
    /// The account alias.
    alias: String,
    /// Public addresses
    #[serde(rename = "publicAddresses")]
    pub(crate) public_addresses: Vec<AccountAddress>,
    /// Internal addresses
    #[serde(rename = "internalAddresses")]
    pub(crate) internal_addresses: Vec<AccountAddress>,
    /// Addresses with unspent outputs
    // used to improve performance for syncing and get balance because it's in most cases only a subset of all addresses
    #[serde(rename = "addressesWithUnspentOutputs")]
    addresses_with_unspent_outputs: Vec<AddressWithUnspentOutputs>,
    /// Outputs
    // stored separated from the account for performance?
    outputs: HashMap<OutputId, OutputData>,
    /// Unspent outputs that are currently used as input for transactions
    // outputs used in transactions should be locked here so they don't get used again, which would result in a
    // conflicting transaction
    #[serde(rename = "lockedOutputs")]
    locked_outputs: HashSet<OutputId>,
    /// Unspent outputs
    // have unspent outputs in a separated hashmap so we don't need to iterate over all outputs we have
    #[serde(rename = "unspentOutputs")]
    unspent_outputs: HashMap<OutputId, OutputData>,
    /// Sent transactions
    // stored separated from the account for performance and only the transaction id here? where to add the network id?
    // transactions: HashSet<TransactionId>,
    transactions: HashMap<TransactionId, types::Transaction>,
    /// Pending transactions
    // Maybe pending transactions even additionally separated?
    #[serde(rename = "pendingTransactions")]
    pending_transactions: HashSet<TransactionId>,
    /// Transaction payloads for received outputs with inputs when not pruned before syncing, can be used to determine
    /// the sender address/es
    #[serde(rename = "incomingTransactions")]
    incoming_transactions: HashMap<TransactionId, (TransactionPayload, Vec<OutputWithMetadataResponse>)>,
    /// Foundries for native tokens in outputs
    #[serde(rename = "nativeTokenFoundries", default)]
    native_token_foundries: HashMap<FoundryId, FoundryOutput>,
}

// #[cfg(test)]
// mod tests {
// use std::fs;
//
// use iota_client::{
// block::{
// address::Address,
// output::{
// dto::OutputDto,
// unlock_condition::{AddressUnlockCondition, UnlockCondition},
// BasicOutputBuilder,
// },
// },
// constants::SHIMMER_COIN_TYPE,
// ClientBuilder,
// };
//
// const TOKEN_SUPPLY: u64 = 1_813_620_509_061_365;
//
// #[tokio::test]
// async fn test_account_balance() {
// std::fs::remove_dir_all("test-storage/message_interface_create_account").unwrap_or(());
// let secret_manager = r#"{"Mnemonic":"acoustic trophy damage hint search taste love bicycle foster cradle brown govern
// endless depend situate athlete pudding blame question genius transfer van random vast"}"#; let client_options = r#"{
// "nodes":[
// {
// "url":"http://localhost:14265/",
// "auth":null,
// "disabled":false
// }
// ]
// }"#;
//
// let options = ManagerOptions {
// #[cfg(feature = "storage")]
// storage_path: Some("test-storage/message_interface_create_account".to_string()),
// client_options: Some(ClientBuilder::new().from_json(client_options).unwrap()),
// coin_type: Some(SHIMMER_COIN_TYPE),
// secret_manager: Some(serde_json::from_str(secret_manager).unwrap()),
// };
//
// let wallet_handle = super::create_message_handler(Some(options)).await.unwrap();
//
// create an account
// let response = message_interface::send_message(
// &wallet_handle,
// Message::CreateAccount {
// alias: None,
// bech32_hrp: None,
// },
// )
// .await
// .expect("No send message response");
//
// match response {
// Response::Account(account) => {
// let id = account.index;
// println!("Created account index: {id}")
// }
// _ => panic!("unexpected response {response:?}"),
// }
//
// std::fs::remove_dir_all("test-storage/message_interface_create_account").unwrap_or(());
// }
// }
