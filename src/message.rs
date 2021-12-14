// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_manager::AccountStore,
    address::{Address, AddressOutput, AddressWrapper, IotaAddress, OutputKind},
    client::ClientOptions,
    event::{emit_transfer_progress, TransferProgressType},
};
use getset::{CopyGetters, Getters, Setters};
use iota_client::common::packable::Packable;

use chrono::prelude::{DateTime, NaiveDateTime, Utc};
pub use iota_client::{
    bee_message::prelude::{
        Essence, IndexationPayload, Input, Message as IotaMessage, MessageId, MigratedFundsEntry, MilestoneIndex,
        MilestonePayload, MilestonePayloadEssence, Output, Parents, Payload, ReceiptPayload, RegularEssence,
        SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput, TailTransactionHash, TransactionPayload,
        TreasuryInput, TreasuryOutput, TreasuryTransactionPayload, UnlockBlock, UtxoInput,
        MILESTONE_MERKLE_PROOF_LENGTH, MILESTONE_PUBLIC_KEY_LENGTH,
    },
    MilestoneResponse,
};
use once_cell::sync::Lazy;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::{
    cmp::Ordering,
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    convert::{TryFrom, TryInto},
    fmt,
    hash::{Hash, Hasher},
    num::NonZeroU64,
    ops::Range,
};
use tokio::sync::RwLock;

// The library do not request a milestone from the node if we have one in the (x / 100, x/ 100 + 100) range
const MILESTONE_CACHE_RANGE: u32 = 100;
/// The node issue a milestone every 10 seconds.
const MILESTONE_ISSUE_RATE_SECS: i64 = 10;

type MilestoneCache = RwLock<HashMap<Range<u32>, MilestoneResponse>>;
fn milestone_cache() -> &'static MilestoneCache {
    static MILESTONE_CACHE: Lazy<MilestoneCache> = Lazy::new(Default::default);
    &MILESTONE_CACHE
}

/// The strategy to use for the remainder value management when sending funds.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "strategy", content = "value")]
pub enum RemainderValueStrategy {
    /// Keep the remainder value on the source address.
    ReuseAddress,
    /// Move the remainder value to a change address.
    ChangeAddress,
    /// Move the remainder value to an address that must belong to the source account.
    #[serde(with = "crate::serde::iota_address_serde")]
    AccountAddress(AddressWrapper),
}

impl Default for RemainderValueStrategy {
    fn default() -> Self {
        Self::ChangeAddress
    }
}

/// Transfer output.
#[derive(Debug, Clone, Deserialize)]
pub struct TransferOutput {
    /// The output value.
    pub amount: NonZeroU64,
    /// The output address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub address: AddressWrapper,
    /// The output type
    #[serde(default = "default_output_kind", rename = "outputKind")]
    pub output_kind: OutputKind,
}

fn default_output_kind() -> OutputKind {
    OutputKind::SignatureLockedSingle
}

impl TransferOutput {
    /// Creates a new transfer output.
    pub fn new(address: AddressWrapper, amount: NonZeroU64, output_kind: Option<OutputKind>) -> Self {
        Self {
            amount,
            address,
            output_kind: output_kind.unwrap_or(OutputKind::SignatureLockedSingle),
        }
    }
}

/// A transfer to make a transaction.
#[derive(Debug, Clone)]
pub struct TransferBuilder {
    /// Transfer outputs.
    outputs: Vec<TransferOutput>,
    /// (Optional) message indexation.
    indexation: Option<IndexationPayload>,
    /// The strategy to use for the remainder value.
    remainder_value_strategy: RemainderValueStrategy,
    /// The input to use (skips input selection)
    input: Option<Vec<(AddressWrapper, Vec<AddressOutput>)>>,
    /// Whether the transfer should emit events or not.
    with_events: bool,
    /// Whether the transfer should skip account syncing or not.
    skip_sync: bool,
}

impl Default for TransferBuilder {
    fn default() -> Self {
        Self {
            outputs: Default::default(),
            indexation: None,
            remainder_value_strategy: RemainderValueStrategy::ChangeAddress,
            input: None,
            with_events: true,
            skip_sync: false,
        }
    }
}

impl<'de> Deserialize<'de> for TransferBuilder {
    fn deserialize<D>(deserializer: D) -> Result<TransferBuilder, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Clone, Deserialize)]
        #[serde(untagged)]
        enum TransactionIndexation {
            String(String),
            Raw(Vec<u8>),
        }
        /// The message's index builder.
        #[derive(Debug, Clone, Deserialize)]
        struct IndexationPayloadBuilder {
            index: TransactionIndexation,
            data: Option<Vec<u8>>,
        }

        impl IndexationPayloadBuilder {
            /// Builds the indexation.
            pub fn finish(self) -> crate::Result<IndexationPayload> {
                let indexation = IndexationPayload::new(
                    &match self.index {
                        TransactionIndexation::String(value) => value.as_bytes().to_vec(),
                        TransactionIndexation::Raw(bytes) => bytes,
                    },
                    &self.data.unwrap_or_default(),
                )?;
                Ok(indexation)
            }
        }

        #[derive(Debug, Clone, Deserialize)]
        pub struct TransferBuilderWrapper {
            /// Single output transfer.
            #[serde(flatten)]
            output: Option<TransferOutput>,
            /// Transfer outputs.
            #[serde(default)]
            outputs: Vec<TransferOutput>,
            /// (Optional) message indexation.
            indexation: Option<IndexationPayloadBuilder>,
            /// The strategy to use for the remainder value.
            remainder_value_strategy: RemainderValueStrategy,
        }

        TransferBuilderWrapper::deserialize(deserializer).and_then(|mut builder| {
            if let Some(output) = builder.output {
                builder.outputs.push(output);
            }
            Ok(TransferBuilder {
                outputs: builder.outputs,
                indexation: match builder.indexation {
                    Some(i) => Some(i.finish().map_err(serde::de::Error::custom)?),
                    None => None,
                },
                remainder_value_strategy: builder.remainder_value_strategy,
                input: None,
                with_events: true,
                skip_sync: false,
            })
        })
    }
}

impl TransferBuilder {
    /// Initialises a new transfer to the given address.
    pub fn new(address: AddressWrapper, amount: NonZeroU64, output_kind: Option<OutputKind>) -> Self {
        Self {
            outputs: vec![TransferOutput {
                amount,
                address,
                output_kind: output_kind.unwrap_or(OutputKind::SignatureLockedSingle),
            }],
            ..Default::default()
        }
    }

    /// Creates a transfer with multiple outputs.
    pub fn with_outputs(outputs: Vec<TransferOutput>) -> crate::Result<Self> {
        if !(1..125).contains(&outputs.len()) {
            return Err(crate::Error::BeeMessage(
                iota_client::bee_message::Error::InvalidInputOutputCount(outputs.len()),
            ));
        }
        Ok(Self {
            outputs,
            ..Default::default()
        })
    }

    /// Sets the remainder value strategy for the transfer.
    pub fn with_remainder_value_strategy(mut self, strategy: RemainderValueStrategy) -> Self {
        self.remainder_value_strategy = strategy;
        self
    }

    /// (Optional) message indexation.
    pub fn with_indexation(mut self, indexation: IndexationPayload) -> Self {
        self.indexation.replace(indexation);
        self
    }

    /// Sets the addresses and utxo to use as transaction input.
    pub(crate) fn with_input(mut self, address: AddressWrapper, inputs: Vec<AddressOutput>) -> Self {
        self.input.replace(vec![(address, inputs)]);
        self
    }

    #[cfg(feature = "participation")]
    /// Sets the utxos to use as transaction input.
    pub(crate) fn with_inputs(mut self, inputs: Vec<AddressOutput>) -> Self {
        let mut address_inputs: HashMap<AddressWrapper, Vec<AddressOutput>> = HashMap::new();

        for input in inputs {
            address_inputs
                .entry(input.address.clone())
                .and_modify(|e| e.push(input.clone()))
                .or_insert_with(|| vec![input]);
        }
        let mut final_address_inputs = Vec::new();
        for value in address_inputs {
            final_address_inputs.push(value);
        }
        self.input.replace(final_address_inputs);
        self
    }

    pub(crate) fn with_events(mut self, flag: bool) -> Self {
        self.with_events = flag;
        self
    }

    /// Skip account syncing before transferring.
    pub fn with_skip_sync(mut self) -> Self {
        self.skip_sync = true;
        self
    }

    /// Builds the transfer.
    pub fn finish(self) -> Transfer {
        Transfer {
            outputs: self.outputs,
            indexation: self.indexation,
            remainder_value_strategy: self.remainder_value_strategy,
            input: self.input,
            with_events: self.with_events,
            skip_sync: self.skip_sync,
        }
    }
}

/// A transfer to make a transaction.
#[derive(Debug, Clone)]
pub struct Transfer {
    /// Transfer outputs.
    pub(crate) outputs: Vec<TransferOutput>,
    /// (Optional) message indexation.
    pub(crate) indexation: Option<IndexationPayload>,
    /// The strategy to use for the remainder value.
    pub(crate) remainder_value_strategy: RemainderValueStrategy,
    /// The addresses to use as input.
    pub(crate) input: Option<Vec<(AddressWrapper, Vec<AddressOutput>)>>,
    /// Whether the transfer should emit events or not.
    pub(crate) with_events: bool,
    /// Whether the transfer should skip account syncing or not.
    pub(crate) skip_sync: bool,
}

impl Transfer {
    /// Initialises the transfer builder.
    pub fn builder(address: AddressWrapper, amount: NonZeroU64, output_kind: Option<OutputKind>) -> TransferBuilder {
        TransferBuilder::new(address, amount, output_kind)
    }

    /// Initialises the transfer builder with multiple outputs.
    pub fn builder_with_outputs(outputs: Vec<TransferOutput>) -> crate::Result<TransferBuilder> {
        TransferBuilder::with_outputs(outputs)
    }

    pub(crate) async fn emit_event_if_needed(&self, account_id: String, event: TransferProgressType) {
        if self.with_events {
            emit_transfer_progress(account_id, event).await;
        }
    }

    pub(crate) fn amount(&self) -> u64 {
        self.outputs.iter().map(|o| o.amount.get()).sum()
    }
}

/// Possible Value units.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ValueUnit {
    /// i
    I,
    /// Ki
    Ki,
    /// Mi
    Mi,
    /// Gi
    Gi,
    /// Ti
    Ti,
    /// Pi
    Pi,
}

impl fmt::Display for ValueUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ValueUnit::I => write!(f, "i"),
            ValueUnit::Ki => write!(f, "Ki"),
            ValueUnit::Mi => write!(f, "Mi"),
            ValueUnit::Gi => write!(f, "Gi"),
            ValueUnit::Ti => write!(f, "Ti"),
            ValueUnit::Pi => write!(f, "Pi"),
        }
    }
}

/// The transaction Value struct.
#[derive(Debug, Getters, Serialize, Deserialize, Clone)]
#[getset(get = "pub")]
pub struct Value {
    /// The value.
    value: u64,
    /// The value's unit.
    unit: ValueUnit,
}

impl Value {
    /// Ititialises a new Value.
    pub fn new(value: u64, unit: ValueUnit) -> Self {
        Self { value, unit }
    }

    /// Formats the value with its unit.
    pub fn with_denomination(&self) -> String {
        format!("{} {}", self.value, self.unit)
    }

    /// The transaction value without its unit.
    pub fn without_denomination(&self) -> u64 {
        let multiplier = match self.unit {
            ValueUnit::I => 1,
            ValueUnit::Ki => 1000,
            ValueUnit::Mi => 1000000,
            ValueUnit::Gi => 1000000000,
            ValueUnit::Ti => 1000000000000,
            ValueUnit::Pi => 1000000000000000,
        };
        self.value * multiplier
    }
}

/// Signature locked single output.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Eq, PartialEq)]
pub struct TransactionSignatureLockedSingleOutput {
    /// The output adrress.
    #[getset(get = "pub")]
    #[serde(with = "crate::serde::iota_address_serde")]
    address: AddressWrapper,
    /// The output amount.
    #[getset(get_copy = "pub")]
    amount: u64,
    /// Whether the output is a remander value output or not.
    #[getset(get_copy = "pub")]
    remainder: bool,
}

impl TransactionSignatureLockedSingleOutput {
    fn new(output: &SignatureLockedSingleOutput, bech32_hrp: String, remainder: bool) -> Self {
        Self {
            address: AddressWrapper::new(*output.address(), bech32_hrp),
            amount: output.amount(),
            remainder,
        }
    }
}

/// Dust allowance output.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Eq, PartialEq)]
#[getset(get = "pub")]
pub struct TransactionSignatureLockedDustAllowanceOutput {
    /// The output adrress.
    #[serde(with = "crate::serde::iota_address_serde")]
    address: AddressWrapper,
    /// The output amount.
    amount: u64,
}

impl TransactionSignatureLockedDustAllowanceOutput {
    fn new(output: &SignatureLockedDustAllowanceOutput, bech32_hrp: String) -> Self {
        Self {
            address: AddressWrapper::new(*output.address(), bech32_hrp),
            amount: output.amount(),
        }
    }
}

/// The transaction output.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum TransactionOutput {
    /// Signature locked single output.
    SignatureLockedSingle(TransactionSignatureLockedSingleOutput),
    /// Dust allowance output.
    SignatureLockedDustAllowance(TransactionSignatureLockedDustAllowanceOutput),
    /// Trasury output.
    Treasury(TreasuryOutput),
}

impl TransactionOutput {
    fn new(output: &Output, bech32_hrp: String, remainder: bool) -> Self {
        match output {
            Output::SignatureLockedSingle(output) => Self::SignatureLockedSingle(
                TransactionSignatureLockedSingleOutput::new(output, bech32_hrp, remainder),
            ),
            Output::SignatureLockedDustAllowance(output) => Self::SignatureLockedDustAllowance(
                TransactionSignatureLockedDustAllowanceOutput::new(output, bech32_hrp),
            ),
            Output::Treasury(output) => Self::Treasury(output.clone()),
        }
    }
}

/// UTXO input.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionUtxoInput {
    /// UTXO input.
    pub input: UtxoInput,
    /// Metadata.
    pub metadata: Option<AddressOutput>,
}

/// Transaction input.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum TransactionInput {
    /// UTXO input.
    Utxo(TransactionUtxoInput),
    /// Treasury input.
    Treasury(TreasuryInput),
}

/// Regular essence.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionRegularEssence {
    inputs: Box<[TransactionInput]>,
    outputs: Box<[TransactionOutput]>,
    payload: Option<Payload>,
    internal: bool,
    pub(crate) incoming: bool,
    value: u64,
    #[serde(rename = "remainderValue")]
    remainder_value: u64,
}

impl TransactionRegularEssence {
    /// Gets the transaction inputs.
    pub fn inputs(&self) -> &[TransactionInput] {
        &self.inputs
    }

    #[allow(dead_code)]
    pub(crate) fn inputs_mut(&mut self) -> &mut [TransactionInput] {
        &mut self.inputs
    }

    /// Gets the transaction outputs.
    pub fn outputs(&self) -> &[TransactionOutput] {
        &self.outputs
    }

    /// Gets the transaction chained payload.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    /// Whether the transaction is between the mnemonic accounts or not.
    pub fn internal(&self) -> bool {
        self.internal
    }

    /// Whether the transaction is incoming or outgoing.
    pub fn incoming(&self) -> bool {
        self.incoming
    }

    /// The transactions's value.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The transactions's remainder value sum.
    pub fn remainder_value(&self) -> u64 {
        self.remainder_value
    }
}

impl TransactionRegularEssence {
    pub(crate) fn is_incoming(&self, account_addresses: &[Address]) -> bool {
        !self.inputs().iter().any(|i| match i {
            crate::message::TransactionInput::Utxo(input) => match input.metadata {
                Some(ref input_metadata) => account_addresses.iter().any(|a| &input_metadata.address == a.address()),
                None => false,
            },
            _ => false,
        })
    }

    async fn new(regular_essence: &RegularEssence, metadata: &TransactionBuilderMetadata<'_>) -> crate::Result<Self> {
        let mut inputs = Vec::new();
        for input in regular_essence.inputs() {
            let input = match input.clone() {
                Input::Utxo(i) => {
                    #[cfg(test)]
                    let metadata: Option<AddressOutput> = None;
                    #[cfg(not(test))]
                    let metadata = {
                        let mut output = None;
                        for address in metadata.account_addresses {
                            if let Some(found_output) = address.outputs().get(i.output_id()) {
                                output.replace(found_output.clone());
                                break;
                            }
                        }
                        if let Some(output) = output {
                            Some(output)
                        } else {
                            let client = crate::client::get_client(metadata.client_options).await?;
                            let client = client.read().await;
                            match client.get_output(&i).await {
                                Ok(output) => {
                                    let output =
                                        AddressOutput::from_output_response(output, metadata.bech32_hrp.clone())?;
                                    Some(output)
                                }
                                Err(iota_client::Error::ResponseError(status_code, _)) if status_code == 404 => None,
                                Err(e) => return Err(e.into()),
                            }
                        }
                    };
                    TransactionInput::Utxo(TransactionUtxoInput { input: i, metadata })
                }
                Input::Treasury(treasury) => TransactionInput::Treasury(treasury),
            };
            inputs.push(input);
        }
        let mut outputs = Vec::new();

        let tx_outputs = regular_essence.outputs();
        match tx_outputs.len() {
            0 => {}
            // if the tx has one output, it is not a remainder output
            1 => {
                outputs.push(TransactionOutput::new(
                    tx_outputs.first().unwrap(),
                    metadata.bech32_hrp.clone(),
                    false,
                ));
            }
            // if the tx has more than one output, we check which one is the remainder
            _ => {
                let tx_outputs: Vec<(&IotaAddress, &Output)> = tx_outputs
                    .iter()
                    .map(|output| {
                        let address = match output {
                            Output::SignatureLockedDustAllowance(o) => o.address(),
                            Output::SignatureLockedSingle(o) => o.address(),
                            _ => unimplemented!(),
                        };
                        (address, output)
                    })
                    .collect();
                // if all outputs belongs to the account, we can't determine whether this transfer is incoming or
                // outgoing; so we assume that the highest address index holds the remainder, and the rest is the
                // transfer outputs
                let all_outputs_belongs_to_account = tx_outputs.iter().all(|(address, _)| {
                    let address_belongs_to_account = metadata
                        .account_addresses
                        .iter()
                        .any(|a| &a.address().as_ref() == address);
                    address_belongs_to_account
                });
                if all_outputs_belongs_to_account {
                    let mut remainder: Option<&Address> = None;
                    for (output_address, _) in &tx_outputs {
                        let account_address = metadata
                            .account_addresses
                            .iter()
                            .find(|a| &a.address().as_ref() == output_address)
                            .unwrap(); // safe to unwrap since we already asserted that the address belongs to the account

                        // if the output is listed on the inputs, it's the remainder output.
                        if inputs.iter().any(|input| match input {
                            TransactionInput::Utxo(input) => {
                                if let Some(metadata) = &input.metadata {
                                    &metadata.address().as_ref() == output_address
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        }) {
                            remainder.replace(account_address);
                            break;
                        }
                        match remainder {
                            Some(remainder_address) => {
                                let address_index = *account_address.key_index();
                                // if the address index is the highest or it's the same as the previous one and this is
                                // a change address, we assume that it holds the remainder value
                                if address_index > *remainder_address.key_index()
                                    || (address_index == *remainder_address.key_index() && *account_address.internal())
                                {
                                    remainder.replace(account_address);
                                }
                            }
                            None => {
                                remainder.replace(account_address);
                            }
                        }
                    }
                    let remainder = remainder.map(|a| a.address().as_ref());
                    for (output_address, output) in tx_outputs {
                        outputs.push(TransactionOutput::new(
                            output,
                            metadata.bech32_hrp.clone(),
                            remainder == Some(output_address),
                        ));
                    }
                } else {
                    let sent = inputs.iter().any(|i| match i {
                        TransactionInput::Utxo(input) => match input.metadata {
                            Some(ref input_metadata) => metadata
                                .account_addresses
                                .iter()
                                .any(|a| &input_metadata.address == a.address()),
                            None => false,
                        },
                        _ => false,
                    });
                    for (output_address, output) in tx_outputs {
                        let address_belongs_to_account = metadata
                            .account_addresses
                            .iter()
                            .any(|a| a.address().as_ref() == output_address);
                        if sent {
                            let remainder = address_belongs_to_account;
                            outputs.push(TransactionOutput::new(output, metadata.bech32_hrp.clone(), remainder));
                        } else {
                            let remainder = !address_belongs_to_account;
                            outputs.push(TransactionOutput::new(output, metadata.bech32_hrp.clone(), remainder));
                        }
                    }
                }
            }
        }

        let mut value = 0;
        let mut remainder_value = 0;
        for output in &outputs {
            let (output_value, remainder) = match output {
                TransactionOutput::SignatureLockedSingle(o) => (o.amount, o.remainder),
                TransactionOutput::SignatureLockedDustAllowance(o) => (o.amount, false),
                TransactionOutput::Treasury(o) => (o.amount(), false),
            };
            if remainder {
                remainder_value += output_value;
            } else {
                value += output_value;
            }
        }

        let mut essence = Self {
            inputs: inputs.into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
            payload: regular_essence.payload().clone(),
            internal: false,
            incoming: false,
            value,
            remainder_value,
        };

        let is_internal = is_internal(
            &essence,
            metadata.accounts.clone(),
            metadata.account_id,
            metadata.account_addresses,
        )
        .await;
        essence.internal = is_internal;
        essence.incoming = essence.is_incoming(metadata.account_addresses);

        Ok(essence)
    }
}

/// The transaction essence.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum TransactionEssence {
    /// Regular essence type.
    Regular(TransactionRegularEssence),
}

impl TransactionEssence {
    #[doc(hidden)]
    pub async fn new(essence: &Essence, metadata: &TransactionBuilderMetadata<'_>) -> crate::Result<Self> {
        let essence = match essence {
            Essence::Regular(regular) => Self::Regular(TransactionRegularEssence::new(regular, metadata).await?),
        };
        Ok(essence)
    }
}

/// A transaction payload.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MessageTransactionPayload {
    essence: TransactionEssence,
    unlock_blocks: Box<[UnlockBlock]>,
}

impl MessageTransactionPayload {
    /// The transaction essence.
    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    pub(crate) fn essence_mut(&mut self) -> &mut TransactionEssence {
        &mut self.essence
    }

    /// The unlock blocks.
    pub fn unlock_blocks(&self) -> &[UnlockBlock] {
        &self.unlock_blocks
    }

    #[doc(hidden)]
    pub async fn new(payload: &TransactionPayload, metadata: &TransactionBuilderMetadata<'_>) -> crate::Result<Self> {
        let mut unlock_blocks = Vec::new();
        for unlock_block in payload.unlock_blocks().as_ref() {
            unlock_blocks.push(unlock_block.clone());
        }
        Ok(Self {
            essence: TransactionEssence::new(payload.essence(), metadata).await?,
            unlock_blocks: unlock_blocks.into_boxed_slice(),
        })
    }
}

/// Milestone payload essence.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Eq, PartialEq)]
pub struct MessageMilestonePayloadEssence {
    /// Milestone index.
    #[getset(get_copy = "pub")]
    index: MilestoneIndex,
    /// Milestone timestamp.
    #[getset(get_copy = "pub")]
    timestamp: u64,
    /// Message parents.
    #[getset(get = "pub")]
    parents: Parents,
    /// Milestone merkle proof.
    #[getset(get = "pub")]
    #[serde(rename = "merkleProof")]
    merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    /// Next PoW score.
    #[getset(get_copy = "pub")]
    #[serde(rename = "nextPowScore")]
    next_pow_score: u32,
    /// Milestone index where the PoW score will change.
    #[getset(get_copy = "pub")]
    #[serde(rename = "nextPowScoreMilestone")]
    next_pow_score_milestone_index: u32,
    /// Milestone public keys.
    #[getset(get = "pub")]
    #[serde(rename = "publicKey")]
    public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    /// Milestone receipt.
    #[getset(get = "pub")]
    receipt: Option<MessagePayload>,
}

impl MessageMilestonePayloadEssence {
    async fn new(essence: &MilestonePayloadEssence, metadata: &TransactionBuilderMetadata<'_>) -> crate::Result<Self> {
        Ok(Self {
            index: essence.index(),
            timestamp: essence.timestamp(),
            parents: essence.parents().clone(),
            merkle_proof: essence.merkle_proof().try_into().unwrap(),
            next_pow_score: essence.next_pow_score(),
            next_pow_score_milestone_index: essence.next_pow_score_milestone_index(),
            public_keys: essence.public_keys().to_vec(),
            receipt: match essence.receipt() {
                Some(p) => {
                    if let Payload::Receipt(receipt) = p {
                        Some(MessagePayload::Receipt(Box::new(MessageReceiptPayload::new(
                            receipt, metadata,
                        ))))
                    } else {
                        None
                    }
                }
                None => None,
            },
        })
    }
}

/// Message milestone payload.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MessageMilestonePayload {
    essence: MessageMilestonePayloadEssence,
    signatures: Vec<Box<[u8]>>,
}

impl MessageMilestonePayload {
    /// The milestone essence.
    pub fn essence(&self) -> &MessageMilestonePayloadEssence {
        &self.essence
    }

    /// The milestone signatures.
    pub fn signatures(&self) -> &Vec<Box<[u8]>> {
        &self.signatures
    }

    #[doc(hidden)]
    pub async fn new(payload: &MilestonePayload, metadata: &TransactionBuilderMetadata<'_>) -> crate::Result<Self> {
        Ok(Self {
            essence: MessageMilestonePayloadEssence::new(payload.essence(), metadata).await?,
            signatures: payload.signatures().to_vec(),
        })
    }
}

/// Tail transaction hash.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MessageTailTransactionHash(TailTransactionHash);

impl<'de> Deserialize<'de> for MessageTailTransactionHash {
    fn deserialize<D>(deserializer: D) -> Result<MessageTailTransactionHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TailTransactionHashOptions {
            Raw(TailTransactionHash),
            Trytes(String),
        }
        let tail = TailTransactionHashOptions::deserialize(deserializer)?;
        let hash = match tail {
            TailTransactionHashOptions::Raw(hash) => MessageTailTransactionHash(hash),
            TailTransactionHashOptions::Trytes(trytes) => {
                let buf = trytes
                    .chars()
                    .map(iota_migration::ternary::Tryte::try_from)
                    .collect::<Result<iota_migration::ternary::TryteBuf, _>>()
                    .map_err(|_| serde::de::Error::custom("invalid tail transaction hash"))?
                    .as_trits()
                    .encode::<iota_migration::ternary::T5B1Buf>();
                MessageTailTransactionHash(
                    TailTransactionHash::new(bytemuck::cast_slice(buf.as_slice().as_i8_slice()).try_into().unwrap())
                        .map_err(|_| serde::de::Error::custom("invalid tail transaction hash"))?,
                )
            }
        };
        Ok(hash)
    }
}

impl Serialize for MessageTailTransactionHash {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(
            &iota_migration::ternary::Trits::<iota_migration::ternary::T5B1>::try_from_raw(
                bytemuck::cast_slice(self.0.as_ref()),
                243,
            )
            .map_err(|_| serde::ser::Error::custom("invalid tail transaction hash"))?
            .to_buf::<iota_migration::ternary::T5B1Buf>()
            .iter_trytes()
            .map(char::from)
            .collect::<String>(),
        )
    }
}

impl AsRef<[u8]> for MessageTailTransactionHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Migrated funds entry.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Eq, PartialEq)]
#[getset(get = "pub")]
pub struct MessageMigratedFundsEntry {
    /// Tail transaction hash.
    #[serde(rename = "tailTransactionHash")]
    tail_transaction_hash: MessageTailTransactionHash,
    /// Output.
    output: TransactionSignatureLockedSingleOutput,
}

impl MessageMigratedFundsEntry {
    #[doc(hidden)]
    pub fn new(entry: &MigratedFundsEntry, metadata: &TransactionBuilderMetadata<'_>) -> Self {
        Self {
            tail_transaction_hash: MessageTailTransactionHash(entry.tail_transaction_hash().clone()),
            output: TransactionSignatureLockedSingleOutput::new(entry.output(), metadata.bech32_hrp.clone(), false),
        }
    }
}

/// Receipt payload.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Eq, PartialEq)]
pub struct MessageReceiptPayload {
    /// Migrated at milestone index.
    #[getset(get_copy = "pub")]
    #[serde(rename = "migratedAt")]
    migrated_at: MilestoneIndex,
    /// Last flag.
    #[getset(get_copy = "pub")]
    last: bool,
    /// Funds.
    #[getset(get = "pub")]
    funds: Vec<MessageMigratedFundsEntry>,
    /// Receipt transaction.
    #[getset(get = "pub")]
    transaction: MessagePayload,
}

impl MessageReceiptPayload {
    #[doc(hidden)]
    pub fn new(payload: &ReceiptPayload, metadata: &TransactionBuilderMetadata<'_>) -> Self {
        Self {
            migrated_at: payload.migrated_at(),
            last: payload.last(),
            funds: payload
                .funds()
                .iter()
                .map(|e| MessageMigratedFundsEntry::new(e, metadata))
                .collect(),
            transaction: if let Payload::TreasuryTransaction(tx) = payload.transaction() {
                MessagePayload::TreasuryTransaction(tx.clone())
            } else {
                unreachable!()
            },
        }
    }
}

/// The message's payload.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    /// Transaction payload.
    Transaction(Box<MessageTransactionPayload>),
    /// Milestone payload.
    Milestone(Box<MessageMilestonePayload>),
    /// Indexation payload.
    Indexation(Box<IndexationPayload>),
    /// Receipt payload.
    Receipt(Box<MessageReceiptPayload>),
    /// Treasury Transaction payload.
    TreasuryTransaction(Box<TreasuryTransactionPayload>),
}

impl MessagePayload {
    pub(crate) async fn new(payload: Payload, metadata: &TransactionBuilderMetadata<'_>) -> crate::Result<Self> {
        let payload = match payload {
            Payload::Transaction(tx) => {
                Self::Transaction(Box::new(MessageTransactionPayload::new(&tx, metadata).await?))
            }
            Payload::Milestone(milestone) => {
                Self::Milestone(Box::new(MessageMilestonePayload::new(&milestone, metadata).await?))
            }
            Payload::Indexation(indexation) => Self::Indexation(indexation),
            Payload::Receipt(receipt) => Self::Receipt(Box::new(MessageReceiptPayload::new(&receipt, metadata))),
            Payload::TreasuryTransaction(treasury_tx) => Self::TreasuryTransaction(treasury_tx),
        };
        Ok(payload)
    }

    pub(crate) fn storage_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        serde_json::to_string(&self).unwrap().hash(&mut hasher);
        hasher.finish()
    }
}

/// A message definition.
#[derive(Debug, Getters, Setters, Clone, Serialize, Deserialize, Eq)]
#[getset(get = "pub", set = "pub(crate)")]
pub struct Message {
    /// The message identifier.
    pub id: MessageId,
    /// The message version.
    pub version: u64,
    /// Message ids this message refers to.
    pub parents: Vec<MessageId>,
    /// Length of the payload.
    #[serde(rename = "payloadLength")]
    pub payload_length: usize,
    /// Message payload.
    pub payload: Option<MessagePayload>,
    /// The transaction timestamp.
    pub timestamp: DateTime<Utc>,
    /// Transaction nonce.
    pub nonce: u64,
    /// Whether the transaction is confirmed or not.
    #[getset(set = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmed: Option<bool>,
    /// Whether the transaction is broadcasted or not.
    #[getset(set = "pub")]
    pub broadcasted: bool,
    /// The message id that reattached this message if any.
    #[serde(rename = "reattachmentMessageId")]
    #[getset(set = "pub(crate)")]
    pub reattachment_message_id: Option<MessageId>,
}

impl Message {
    pub(crate) fn set_bech32_hrp(&mut self, bech32_hrp: String) {
        if let Some(MessagePayload::Transaction(tx)) = self.payload.as_mut() {
            match tx.essence_mut() {
                TransactionEssence::Regular(essence) => {
                    for output in essence.outputs.iter_mut() {
                        match output {
                            TransactionOutput::SignatureLockedSingle(output) => {
                                output.address.bech32_hrp = bech32_hrp.clone();
                            }
                            TransactionOutput::SignatureLockedDustAllowance(output) => {
                                output.address.bech32_hrp = bech32_hrp.clone();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn is_remainder(&self, address: &AddressWrapper) -> Option<bool> {
        if let Some(MessagePayload::Transaction(tx)) = &self.payload {
            match tx.essence() {
                TransactionEssence::Regular(essence) => {
                    for output in essence.outputs() {
                        let (output_address, remainder) = match output {
                            TransactionOutput::SignatureLockedSingle(o) => (o.address(), o.remainder),
                            TransactionOutput::SignatureLockedDustAllowance(o) => (o.address(), false),
                            _ => unimplemented!(),
                        };
                        if output_address == address {
                            return Some(remainder);
                        }
                    }
                }
            }
        }
        None
    }
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.as_ref().cmp(other.id.as_ref())
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn transaction_inputs_belonging_to_account(
    essence: &TransactionRegularEssence,
    account_addresses: &[Address],
) -> Vec<TransactionInput> {
    let mut inputs = Vec::new();
    for input in essence.inputs() {
        if let TransactionInput::Utxo(i) = input {
            if let Some(metadata) = &i.metadata {
                if account_addresses
                    .iter()
                    .any(|address| address.address() == &metadata.address)
                {
                    inputs.push(input.clone());
                }
            }
        }
    }
    inputs
}

fn transaction_outputs_belonging_to_account(
    essence: &TransactionRegularEssence,
    account_addresses: &[Address],
) -> Vec<TransactionOutput> {
    let mut outputs = Vec::new();
    for output in essence.outputs() {
        let output_address = match output {
            TransactionOutput::SignatureLockedDustAllowance(o) => o.address(),
            TransactionOutput::SignatureLockedSingle(o) => o.address(),
            _ => unimplemented!(),
        };
        if account_addresses
            .iter()
            .any(|address| address.address() == output_address)
        {
            outputs.push(output.clone());
        }
    }
    outputs
}

async fn is_internal(
    essence: &TransactionRegularEssence,
    accounts: AccountStore,
    account_id: &str,
    account_addresses: &[Address],
) -> bool {
    let mut inputs_belonging_to_account = Vec::new();
    let mut outputs_belonging_to_account = Vec::new();
    for (id, account_handle) in accounts.read().await.iter() {
        if id == account_id {
            inputs_belonging_to_account.extend(transaction_inputs_belonging_to_account(essence, account_addresses));
            outputs_belonging_to_account.extend(transaction_outputs_belonging_to_account(essence, account_addresses));
        } else {
            let account = account_handle.read().await;
            inputs_belonging_to_account.extend(transaction_inputs_belonging_to_account(essence, account.addresses()));
            outputs_belonging_to_account.extend(transaction_outputs_belonging_to_account(essence, account.addresses()));
        }

        if essence.inputs().iter().all(|i| inputs_belonging_to_account.contains(i))
            && essence
                .outputs()
                .iter()
                .all(|o| outputs_belonging_to_account.contains(o))
        {
            return true;
        }
    }
    false
}

#[doc(hidden)]
pub struct TransactionBuilderMetadata<'a> {
    pub id: &'a MessageId,
    pub bech32_hrp: String,
    pub accounts: AccountStore,
    pub account_id: &'a str,
    pub account_addresses: &'a [Address],
    pub client_options: &'a ClientOptions,
}

pub(crate) struct MessageBuilder<'a> {
    id: MessageId,
    iota_message: IotaMessage,
    accounts: AccountStore,
    account_id: &'a str,
    account_addresses: &'a [Address],
    confirmed: Option<bool>,
    bech32_hrp: String,
    client_options: &'a ClientOptions,
}

impl<'a> MessageBuilder<'a> {
    pub fn new(
        id: MessageId,
        iota_message: IotaMessage,
        accounts: AccountStore,
        account_id: &'a str,
        account_addresses: &'a [Address],
        bech32_hrp: String,
        client_options: &'a ClientOptions,
    ) -> Self {
        Self {
            id,
            iota_message,
            accounts,
            account_id,
            account_addresses,
            confirmed: None,
            bech32_hrp,
            client_options,
        }
    }

    pub fn with_confirmed(mut self, confirmed: Option<bool>) -> Self {
        self.confirmed = confirmed;
        self
    }

    pub async fn finish(self) -> crate::Result<Message> {
        let packed_payload = self.iota_message.payload().pack_new();

        let payload = match self.iota_message.payload() {
            Some(payload) => Some(
                MessagePayload::new(
                    payload.clone(),
                    &TransactionBuilderMetadata {
                        id: &self.id,
                        bech32_hrp: self.bech32_hrp.clone(),
                        accounts: self.accounts.clone(),
                        account_id: self.account_id,
                        account_addresses: self.account_addresses,
                        client_options: self.client_options,
                    },
                )
                .await?,
            ),
            None => None,
        };

        let mut timestamp = Utc::now();
        let client_guard = crate::client::get_client(self.client_options).await?;
        let client = client_guard.read().await;
        if let Ok(metadata) = client.get_message().metadata(&self.id).await {
            timestamp = match metadata.referenced_by_milestone_index {
                Some(ms_index) => {
                    let mut date_time = Utc::now();
                    let initial = ms_index / MILESTONE_CACHE_RANGE;
                    let range = initial..initial + MILESTONE_CACHE_RANGE;
                    match milestone_cache().write().await.entry(range) {
                        Entry::Vacant(entry) => {
                            if let Ok(milestone) = client.get_milestone(ms_index).await {
                                date_time = DateTime::from_utc(
                                    NaiveDateTime::from_timestamp(milestone.timestamp as i64, 0),
                                    Utc,
                                );
                                entry.insert(milestone);
                            }
                        }
                        Entry::Occupied(entry) => {
                            let milestone = *entry.get();
                            let index_diff = ms_index as i64 - milestone.index as i64;
                            let approx_timestamp =
                                milestone.timestamp as i64 + (index_diff * MILESTONE_ISSUE_RATE_SECS);
                            date_time = DateTime::from_utc(NaiveDateTime::from_timestamp(approx_timestamp, 0), Utc);
                        }
                    }
                    date_time
                }
                _ => Utc::now(),
            };
        }

        let message = Message {
            id: self.id,
            version: 1,
            parents: (*self.iota_message.parents()).to_vec(),
            payload_length: packed_payload.len(),
            payload,
            timestamp,
            nonce: self.iota_message.nonce(),
            confirmed: self.confirmed,
            broadcasted: true,
            reattachment_message_id: None,
        };
        Ok(message)
    }
}

impl Message {
    pub(crate) fn from_iota_message<'a>(
        id: MessageId,
        iota_message: IotaMessage,
        accounts: AccountStore,
        account_id: &'a str,
        account_addresses: &'a [Address],
        client_options: &'a ClientOptions,
    ) -> MessageBuilder<'a> {
        MessageBuilder::new(
            id,
            iota_message,
            accounts,
            account_id,
            account_addresses,
            account_addresses
                .iter()
                .next()
                .expect("No address in account")
                .address()
                .bech32_hrp()
                .to_string(),
            client_options,
        )
    }

    /// The message's addresses.
    pub fn addresses(&self) -> Vec<&AddressWrapper> {
        match &self.payload {
            Some(MessagePayload::Transaction(tx)) => match tx.essence() {
                TransactionEssence::Regular(essence) => essence
                    .outputs()
                    .iter()
                    .map(|output| match output {
                        TransactionOutput::SignatureLockedDustAllowance(o) => o.address(),
                        TransactionOutput::SignatureLockedSingle(o) => o.address(),
                        _ => unimplemented!(),
                    })
                    .collect(),
            },
            _ => vec![],
        }
    }
}

/// Message type.
#[derive(Debug, Clone, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    /// Message received.
    Received = 1,
    /// Message sent.
    Sent = 2,
    /// Message not broadcasted.
    Failed = 3,
    /// Message not confirmed.
    Unconfirmed = 4,
    /// A value message.
    Value = 5,
    /// Message confirmed.
    Confirmed = 6,
}
