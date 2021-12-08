// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::Account,
    message::{Message, MessagePayload, TransactionEssence, TransactionInput},
    signing::GenerateAddressMetadata,
};
use getset::{Getters, Setters};
pub use iota_client::bee_message::prelude::{Address as IotaAddress, Ed25519Address, Input, UtxoInput};
use iota_client::{
    bee_message::prelude::{MessageId, OutputId, TransactionId},
    bee_rest_api::types::{
        dtos::{AddressDto, OutputDto},
        responses::OutputResponse,
    },
};
use serde::{ser::Serializer, Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::TryInto,
    hash::{Hash, Hasher},
    str::FromStr,
};

/// The address output kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputKind {
    /// SignatureLockedSingle output.
    SignatureLockedSingle,
    /// Dust allowance output.
    SignatureLockedDustAllowance,
    /// Treasury output.
    Treasury,
}

impl FromStr for OutputKind {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "SignatureLockedSingle" => Self::SignatureLockedSingle,
            "SignatureLockedDustAllowance" => Self::SignatureLockedDustAllowance,
            "Treasury" => Self::Treasury,
            _ => return Err(crate::Error::InvalidOutputKind(s.to_string())),
        };
        Ok(kind)
    }
}

/// An Address output.
#[derive(Debug, Getters, Setters, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct AddressOutput {
    /// Transaction ID of the output
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId,
    /// Message ID of the output
    #[serde(rename = "messageId")]
    pub message_id: MessageId,
    /// Output index.
    pub index: u16,
    /// Output amount.
    pub amount: u64,
    /// Spend status of the output,
    #[serde(rename = "isSpent")]
    #[getset(set = "pub(crate)")]
    pub is_spent: bool,
    /// Associated address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub address: AddressWrapper,
    /// Output kind.
    pub kind: OutputKind,
}

impl AddressOutput {
    /// The output identifier.
    pub fn id(&self) -> crate::Result<OutputId> {
        OutputId::new(self.transaction_id, self.index).map_err(Into::into)
    }

    /// Checks if the output is referenced on a pending message or a confirmed message
    pub(crate) fn is_used(&self, messages: &[Message]) -> bool {
        let output_id = UtxoInput::new(self.transaction_id, self.index).unwrap();
        messages.iter().any(|m| {
            // message is pending or confirmed
            if m.confirmed().unwrap_or(true) {
                match m.payload() {
                    Some(MessagePayload::Transaction(tx)) => match tx.essence() {
                        TransactionEssence::Regular(essence) => essence.inputs().iter().any(|input| {
                            if let TransactionInput::Utxo(x) = input {
                                x.input == output_id
                            } else {
                                false
                            }
                        }),
                    },
                    _ => false,
                }
            } else {
                false
            }
        })
    }

    pub(crate) fn from_output_response(output: OutputResponse, bech32_hrp: String) -> crate::Result<Self> {
        let (address, amount, kind) = match output.output {
            OutputDto::SignatureLockedSingle(output) => {
                let address = match output.address {
                    AddressDto::Ed25519(ed25519_address) => IotaAddress::Ed25519(Ed25519Address::new(
                        hex::decode(ed25519_address.address)
                            .map_err(|_| crate::Error::InvalidAddress)?
                            .try_into()
                            .map_err(|_| crate::Error::InvalidAddressLength)?,
                    )),
                };
                (address, output.amount, OutputKind::SignatureLockedSingle)
            }
            OutputDto::SignatureLockedDustAllowance(output) => {
                let address = match output.address {
                    AddressDto::Ed25519(ed25519_address) => IotaAddress::Ed25519(Ed25519Address::new(
                        hex::decode(ed25519_address.address)
                            .map_err(|_| crate::Error::InvalidAddress)?
                            .try_into()
                            .map_err(|_| crate::Error::InvalidAddressLength)?,
                    )),
                };
                (address, output.amount, OutputKind::SignatureLockedDustAllowance)
            }
            OutputDto::Treasury(output) => (
                // dummy address
                IotaAddress::Ed25519(Ed25519Address::new([0; 32])),
                output.amount,
                OutputKind::Treasury,
            ),
        };
        let output = Self {
            transaction_id: TransactionId::new(
                hex::decode(output.transaction_id).map_err(|_| crate::Error::InvalidTransactionId)?[..]
                    .try_into()
                    .map_err(|_| crate::Error::InvalidTransactionId)?,
            ),
            message_id: MessageId::new(
                hex::decode(output.message_id).map_err(|_| crate::Error::InvalidMessageId)?[..]
                    .try_into()
                    .map_err(|_| crate::Error::InvalidMessageId)?,
            ),
            index: output.output_index,
            amount,
            is_spent: output.is_spent,
            address: AddressWrapper::new(address, bech32_hrp),
            kind,
        };
        Ok(output)
    }
}

/// The address builder.
#[derive(Default)]
pub struct AddressBuilder {
    address: Option<AddressWrapper>,
    key_index: Option<usize>,
    internal: bool,
    outputs: Option<Vec<AddressOutput>>,
}

impl AddressBuilder {
    /// Initialises a new instance of the address builder.
    pub fn new() -> AddressBuilder {
        Default::default()
    }

    /// Defines the address.
    pub fn address(mut self, address: AddressWrapper) -> Self {
        self.address.replace(address);
        self
    }

    /// Sets the address key index.
    pub fn key_index(mut self, key_index: usize) -> Self {
        self.key_index.replace(key_index);
        self
    }

    /// Sets the address outputs.
    pub fn outputs(mut self, outputs: Vec<AddressOutput>) -> Self {
        self.outputs.replace(outputs);
        self
    }

    /// Sets the `internal` flag.
    pub fn internal(mut self, internal: bool) -> Self {
        self.internal = internal;
        self
    }

    /// Builds the address.
    pub fn build(self) -> crate::Result<Address> {
        let iota_address = self.address.ok_or(crate::Error::AddressBuildRequiredField(
            crate::error::AddressBuildRequiredField::Address,
        ))?;
        let address_outputs = self.outputs.ok_or(crate::Error::AddressBuildRequiredField(
            crate::error::AddressBuildRequiredField::Outputs,
        ))?;
        let mut outputs = HashMap::new();
        for output in address_outputs {
            outputs.insert(output.id()?, output);
        }
        let address = Address {
            address: iota_address,
            key_index: self.key_index.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::KeyIndex,
            ))?,
            internal: self.internal,
            outputs,
        };
        Ok(address)
    }
}

/// An address and its network type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressWrapper {
    pub(crate) inner: IotaAddress,
    pub(crate) bech32_hrp: String,
}

impl AsRef<IotaAddress> for AddressWrapper {
    fn as_ref(&self) -> &IotaAddress {
        &self.inner
    }
}

impl AddressWrapper {
    /// Create a new address wrapper.
    pub fn new(address: IotaAddress, bech32_hrp: String) -> Self {
        Self {
            inner: address,
            bech32_hrp,
        }
    }

    /// Encodes the address as bech32.
    pub fn to_bech32(&self) -> String {
        self.inner.to_bech32(&self.bech32_hrp)
    }

    pub(crate) fn bech32_hrp(&self) -> &str {
        &self.bech32_hrp
    }
}

/// An address.
#[derive(Debug, Getters, Setters, Clone, Eq, Deserialize)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub(crate) address: AddressWrapper,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    pub(crate) key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    pub(crate) internal: bool,
    /// The address outputs.
    #[getset(set = "pub(crate)")]
    pub(crate) outputs: HashMap<OutputId, AddressOutput>,
}

impl Serialize for Address {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct AddressDto<'a> {
            #[serde(with = "crate::serde::iota_address_serde")]
            address: &'a AddressWrapper,
            balance: u64,
            #[serde(rename = "keyIndex")]
            key_index: usize,
            internal: bool,
            outputs: &'a HashMap<OutputId, AddressOutput>,
        }
        let address = AddressDto {
            address: &self.address,
            balance: self.balance(),
            key_index: self.key_index,
            internal: self.internal,
            outputs: &self.outputs,
        };
        address.serialize(s)
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.to_bech32().cmp(&other.address.to_bech32())
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.to_bech32().hash(state);
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.address.to_bech32() == other.address.to_bech32()
    }
}

impl Address {
    /// Gets a new instance of the address builder.
    #[doc(hidden)]
    pub fn builder() -> AddressBuilder {
        AddressBuilder::new()
    }

    pub(crate) fn available_outputs(&self, sent_messages: &[Message]) -> Vec<&AddressOutput> {
        self.outputs
            .values()
            .filter(|o| !(o.is_spent || o.is_used(sent_messages)))
            .collect()
    }

    /// Address total balance
    pub fn balance(&self) -> u64 {
        self.outputs
            .values()
            .fold(0, |acc, o| acc + if o.is_spent { 0 } else { *o.amount() })
    }

    pub(crate) fn available_balance(&self, sent_messages: &[Message]) -> u64 {
        self.available_outputs(sent_messages)
            .iter()
            .fold(0, |acc, o| acc + *o.amount())
    }

    pub(crate) fn outputs_mut(&mut self) -> &mut HashMap<OutputId, AddressOutput> {
        &mut self.outputs
    }

    /// Updates the Bech32 human readable part.
    #[doc(hidden)]
    pub fn set_bech32_hrp(&mut self, hrp: String) {
        self.address.bech32_hrp = hrp.to_string();
        for output in self.outputs.values_mut() {
            output.address.bech32_hrp = hrp.to_string();
        }
    }
}

/// Parses a bech32 address string.
pub fn parse<A: AsRef<str>>(address: A) -> crate::Result<AddressWrapper> {
    let address = address.as_ref();
    let mut tokens = address.split('1');
    let hrp = tokens.next().ok_or(crate::Error::InvalidAddress)?;
    let address = iota_client::bee_message::address::Address::try_from_bech32(address)?;
    Ok(AddressWrapper::new(address, hrp.to_string()))
}

pub(crate) async fn get_iota_address(
    account: &Account,
    address_index: usize,
    internal: bool,
    bech32_hrp: String,
    metadata: GenerateAddressMetadata,
) -> crate::Result<AddressWrapper> {
    let signer = crate::signing::get_signer(account.signer_type()).await;
    let mut signer = signer.lock().await;
    let address = signer
        .generate_address(account, address_index, internal, metadata)
        .await?;
    Ok(AddressWrapper::new(address, bech32_hrp))
}

/// Gets an unused public address for the given account.
pub(crate) async fn get_new_address(account: &Account, metadata: GenerateAddressMetadata) -> crate::Result<Address> {
    let key_index = account.addresses().iter().filter(|a| !a.internal()).count();
    let bech32_hrp = match account.addresses().first() {
        Some(address) => address.address().bech32_hrp().to_string(),
        None => {
            crate::client::get_client(account.client_options())
                .await?
                .read()
                .await
                .get_network_info()
                .await?
                .bech32_hrp
        }
    };
    let iota_address = get_iota_address(account, key_index, false, bech32_hrp, metadata).await?;
    let address = Address {
        address: iota_address,
        key_index,
        internal: false,
        outputs: Default::default(),
    };
    Ok(address)
}

/// Gets a public address for the given account.
pub(crate) async fn get_address_with_index(
    account: &Account,
    key_index: usize,
    bech32_hrp: String,
    metadata: GenerateAddressMetadata,
) -> crate::Result<Address> {
    let iota_address = get_iota_address(account, key_index, false, bech32_hrp, metadata).await?;
    let address = Address {
        address: iota_address,
        key_index,
        internal: false,
        outputs: Default::default(),
    };
    Ok(address)
}

/// Gets an unused change address for the given account and address.
pub(crate) async fn get_new_change_address(
    account: &Account,
    key_index: usize,
    bech32_hrp: String,
    metadata: GenerateAddressMetadata,
) -> crate::Result<Address> {
    let iota_address = get_iota_address(account, key_index, true, bech32_hrp, metadata).await?;
    let address = Address {
        address: iota_address,
        key_index,
        internal: true,
        outputs: Default::default(),
    };
    Ok(address)
}

pub(crate) fn is_unspent(sent_messages: &[Message], address: &AddressWrapper) -> bool {
    !sent_messages
        .iter()
        .any(|message| message.addresses().contains(&address))
}

#[cfg(test)]
mod tests {
    use crate::message::MessageType;

    #[tokio::test]
    async fn is_unspent_false() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        let account_address = account_handle.generate_address().await.unwrap();
        let address = crate::test_utils::generate_random_address();
        let spent_tx = crate::test_utils::GenerateMessageBuilder::default()
            .address(address.clone())
            .input_address(Some(account_address.address().clone()))
            .account_addresses(account_handle.addresses().await)
            .build()
            .await;

        account_handle
            .write()
            .await
            .save_messages(vec![spent_tx])
            .await
            .unwrap();

        let response = super::is_unspent(
            &account_handle
                .list_messages(0, 0, Some(MessageType::Sent))
                .await
                .unwrap(),
            address.address(),
        );
        assert!(!response);
    }

    #[tokio::test]
    async fn is_unspent_true() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        let address = crate::test_utils::generate_random_iota_address();

        let response = super::is_unspent(
            &account_handle
                .list_messages(0, 0, Some(MessageType::Sent))
                .await
                .unwrap(),
            &address,
        );
        assert!(response);
    }
}
