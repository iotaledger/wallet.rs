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

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use getset::{Getters, Setters};
use iota_client::{
    api_types::response::OutputWithMetadataResponse,
    block::{
        output::{FoundryId, FoundryOutput, OutputId},
        payload::{
            transaction::{TransactionEssence, TransactionId},
            TransactionPayload,
        },
        BlockId,
    },
};
use serde::{de, Deserialize, Deserializer, Serialize};

use self::types::{
    address::{AccountAddress, AddressWithUnspentOutputs},
    AccountBalance, OutputData, Transaction,
};
pub use self::{
    handle::AccountHandle,
    operations::{
        address_generation::AddressGenerationOptions,
        output_claiming::OutputsToClaim,
        syncing::SyncOptions,
        transaction::{
            prepare_output::{Assets, Features, OutputOptions, StorageDeposit},
            RemainderValueStrategy, TransactionOptions,
        },
    },
    types::OutputDataDto,
};
use crate::account::types::InclusionState;

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
    #[serde(deserialize_with = "deserialize_or_convert")]
    incoming_transactions: HashMap<TransactionId, Transaction>,
    /// Foundries for native tokens in outputs
    #[serde(rename = "nativeTokenFoundries", default)]
    native_token_foundries: HashMap<FoundryId, FoundryOutput>,
}

#[test]
fn serialize() {
    use iota_client::block::{
        address::{Address, Ed25519Address},
        input::{Input, UtxoInput},
        output::{unlock_condition::AddressUnlockCondition, BasicOutput, InputsCommitment, Output},
        payload::{
            transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
            TransactionPayload,
        },
        protocol::ProtocolParameters,
        signature::{Ed25519Signature, Signature},
        unlock::{ReferenceUnlock, SignatureUnlock, Unlock, Unlocks},
    };

    const TRANSACTION_ID: &str = "0x24a1f46bdb6b2bf38f1c59f73cdd4ae5b418804bb231d76d06fbf246498d5883";
    const ED25519_ADDRESS: &str = "0xe594f9a895c0e0a6760dd12cffc2c3d1e1cbf7269b328091f96ce3d0dd550b75";
    const ED25519_PUBLIC_KEY: &str = "0x1da5ddd11ba3f961acab68fafee3177d039875eaa94ac5fdbff8b53f0c50bfb9";
    const ED25519_SIGNATURE: &str = "0xc6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

    let protocol_parameters = ProtocolParameters::new(
        2,
        String::from("testnet"),
        String::from("rms"),
        1500,
        15,
        iota_client::block::output::RentStructure::new(500, 10, 1),
        1_813_620_509_061_365,
    )
    .unwrap();

    let transaction_id = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(transaction_id, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(transaction_id, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .unwrap()
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish(protocol_parameters.token_supply())
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(protocol_parameters.network_id(), InputsCommitment::from([0u8; 32]))
            .with_inputs(vec![input1, input2])
            .add_output(output)
            .finish(&protocol_parameters)
            .unwrap(),
    );

    let pub_key_bytes: [u8; 32] = prefix_hex::decode(ED25519_PUBLIC_KEY).unwrap();
    let sig_bytes: [u8; 64] = prefix_hex::decode(ED25519_SIGNATURE).unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock = Unlock::Signature(SignatureUnlock::from(Signature::Ed25519(signature)));
    let ref_unlock = Unlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlocks = Unlocks::new(vec![sig_unlock, ref_unlock]).unwrap();

    let tx_payload = TransactionPayload::new(essence, unlocks).unwrap();

    let incoming_transaction = Transaction {
        transaction_id: TransactionId::from_str("0x131fc4cb8f315ae36ae3bf6a4e4b3486d5f17581288f1217410da3e0700d195a")
            .unwrap(),
        payload: tx_payload,
        block_id: None,
        network_id: 0,
        timestamp: 0,
        inclusion_state: InclusionState::Pending,
        incoming: false,
        note: None,
        inputs: Vec::new(),
    };

    let mut incoming_transactions = HashMap::new();
    incoming_transactions.insert(
        TransactionId::from_str("0x131fc4cb8f315ae36ae3bf6a4e4b3486d5f17581288f1217410da3e0700d195a").unwrap(),
        incoming_transaction,
    );

    let account = Account {
        index: 0,
        coin_type: 4218,
        alias: "0".to_string(),
        public_addresses: Vec::new(),
        internal_addresses: Vec::new(),
        addresses_with_unspent_outputs: Vec::new(),
        outputs: HashMap::new(),
        locked_outputs: HashSet::new(),
        unspent_outputs: HashMap::new(),
        transactions: HashMap::new(),
        pending_transactions: HashSet::new(),
        incoming_transactions,
        native_token_foundries: HashMap::new(),
    };

    serde_json::from_str::<Account>(&serde_json::to_string(&account).unwrap()).unwrap();
}

// Custom deserialization to stay backwards compatible
fn deserialize_or_convert<'de, D>(deserializer: D) -> Result<HashMap<TransactionId, Transaction>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OldOrNew {
        New(HashMap<TransactionId, Transaction>),
        Old(HashMap<TransactionId, (TransactionPayload, Vec<OutputWithMetadataResponse>)>),
    }

    // This works
    // type TestType = HashMap<TransactionId, Transaction>;
    // return Ok(TestType::deserialize(deserializer)?);

    Ok(match OldOrNew::deserialize(deserializer)? {
        OldOrNew::New(v) => v,
        OldOrNew::Old(v) => {
            let mut new = HashMap::new();
            for (tx_id, (tx_payload, inputs)) in v {
                new.insert(
                    tx_id,
                    build_transaction_from_payload_and_inputs(tx_id, tx_payload, inputs).map_err(de::Error::custom)?,
                );
            }
            new
        }
    })
}

pub(crate) fn build_transaction_from_payload_and_inputs(
    tx_id: TransactionId,
    tx_payload: TransactionPayload,
    inputs: Vec<OutputWithMetadataResponse>,
) -> crate::Result<Transaction> {
    let TransactionEssence::Regular(tx_essence) = &tx_payload.essence();
    Ok(Transaction {
        payload: tx_payload.clone(),
        block_id: inputs
            .first()
            .and_then(|i| BlockId::from_str(&i.metadata.block_id).ok()),
        inclusion_state: InclusionState::Confirmed,
        timestamp: inputs
            .first()
            .and_then(|i| i.metadata.milestone_timestamp_spent.map(|t| t as u128 * 1000))
            .unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time went backwards")
                    .as_millis()
            }),
        transaction_id: tx_id,
        network_id: tx_essence.network_id(),
        incoming: true,
        note: None,
        inputs,
    })
}
