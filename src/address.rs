// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, message::MessageType, signing::GenerateAddressMetadata};
use bee_rest_api::{
    handlers::output::OutputResponse,
    types::{AddressDto, OutputDto},
};
use getset::{Getters, Setters};
use iota::message::prelude::{MessageId, TransactionId};
pub use iota::{Address as IotaAddress, Ed25519Address, Input, Payload, UTXOInput};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
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

/// An Address output.
#[derive(Debug, Getters, Setters, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct AddressOutput {
    /// Transaction ID of the output
    #[serde(rename = "transactionId")]
    pub(crate) transaction_id: TransactionId,
    /// Message ID of the output
    #[serde(rename = "messageId")]
    pub(crate) message_id: MessageId,
    /// Output index.
    pub(crate) index: u16,
    /// Output amount.
    pub(crate) amount: u64,
    /// Spend status of the output,
    #[serde(rename = "isSpent")]
    pub(crate) is_spent: bool,
    /// Associated address.
    #[serde(with = "crate::serde::iota_address_serde")]
    pub(crate) address: AddressWrapper,
    /// Output kind.
    pub(crate) kind: OutputKind,
}

impl AddressOutput {
    /// Checks if the output is referenced on a pending message or a confirmed message
    pub(crate) fn is_used(&self, account: &Account) -> bool {
        let output_id = UTXOInput::new(self.transaction_id, self.index).unwrap();
        account.list_messages(0, 0, Some(MessageType::Sent)).iter().any(|m| {
            // message is pending or confirmed
            if m.confirmed().unwrap_or(true) {
                match m.payload() {
                    Some(Payload::Transaction(tx)) => tx.essence().inputs().iter().any(|input| {
                        if let Input::UTXO(x) = input {
                            x == &output_id
                        } else {
                            false
                        }
                    }),
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
pub(crate) struct AddressBuilder {
    address: Option<AddressWrapper>,
    balance: Option<u64>,
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
        self.address = Some(address);
        self
    }

    /// Sets the address balance.
    pub fn balance(mut self, balance: u64) -> Self {
        self.balance = Some(balance);
        self
    }

    /// Sets the address key index.
    pub fn key_index(mut self, key_index: usize) -> Self {
        self.key_index = Some(key_index);
        self
    }

    /// Sets the address outputs.
    pub fn outputs(mut self, outputs: Vec<AddressOutput>) -> Self {
        self.outputs = Some(outputs);
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
        let address = Address {
            address: iota_address,
            balance: self.balance.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::Balance,
            ))?,
            key_index: self.key_index.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::KeyIndex,
            ))?,
            internal: self.internal,
            outputs: self.outputs.ok_or(crate::Error::AddressBuildRequiredField(
                crate::error::AddressBuildRequiredField::Outputs,
            ))?,
        };
        Ok(address)
    }
}

/// An address and its network type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressWrapper {
    inner: IotaAddress,
    bech32_hrp: String,
}

impl AsRef<IotaAddress> for AddressWrapper {
    fn as_ref(&self) -> &IotaAddress {
        &self.inner
    }
}

impl AddressWrapper {
    pub(crate) fn new(address: IotaAddress, bech32_hrp: String) -> Self {
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
#[derive(Debug, Getters, Setters, Clone, Eq, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Address {
    /// The address.
    #[serde(with = "crate::serde::iota_address_serde")]
    address: AddressWrapper,
    /// The address balance.
    #[getset(set = "pub")]
    balance: u64,
    /// The address key index.
    #[serde(rename = "keyIndex")]
    #[getset(set = "pub(crate)")]
    key_index: usize,
    /// Determines if an address is a public or an internal (change) address.
    #[getset(set = "pub(crate)")]
    internal: bool,
    /// The address outputs.
    #[getset(set = "pub(crate)")]
    pub(crate) outputs: Vec<AddressOutput>,
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
    pub(crate) fn handle_new_output(&mut self, output: AddressOutput) {
        if !self.outputs.iter().any(|o| o == &output) {
            let spent_existing_output = self.outputs.iter().position(|o| {
                o.message_id == output.message_id
                    && o.transaction_id == output.transaction_id
                    && o.index == output.index
                    && o.amount == output.amount
                    && (!o.is_spent && output.is_spent)
            });
            if let Some(spent_output) = spent_existing_output {
                log::debug!("[ADDRESS] got spent of {:?}", spent_output);
                self.balance -= output.amount;
                self.outputs.remove(spent_output);
            } else {
                log::debug!("[ADDRESS] got new output {:?}", output);
                self.balance += output.amount;
                self.outputs.push(output);
            }
        }
    }

    /// Gets the list of outputs that aren't spent or pending.
    pub fn available_outputs(&self, account: &Account) -> Vec<&AddressOutput> {
        self.outputs.iter().filter(|o| !o.is_used(account)).collect()
    }

    pub(crate) fn available_balance(&self, account: &Account) -> u64 {
        self.available_outputs(account)
            .iter()
            .fold(0, |acc, o| acc + *o.amount())
    }

    pub(crate) fn set_bech32_hrp(&mut self, hrp: String) {
        self.address.bech32_hrp = hrp.to_string();
        for output in self.outputs.iter_mut() {
            output.address.bech32_hrp = hrp.to_string();
        }
    }
}

/// Parses a bech32 address string.
pub fn parse<A: AsRef<str>>(address: A) -> crate::Result<AddressWrapper> {
    let address = address.as_ref();
    let mut tokens = address.split('1');
    let hrp = tokens.next().unwrap();
    let address = iota::Address::try_from_bech32(address).or_else(|_| {
        if let Ok(ed25519_address) = Ed25519Address::from_str(address) {
            Ok(IotaAddress::Ed25519(ed25519_address))
        } else {
            Err(crate::Error::InvalidAddress)
        }
    });
    Ok(AddressWrapper::new(address?, hrp.to_string()))
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
        .generate_address(&account, address_index, internal, metadata)
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
                .await
                .read()
                .await
                .get_network_info()
                .bech32_hrp
        }
    };
    let iota_address = get_iota_address(&account, key_index, false, bech32_hrp, metadata).await?;
    let address = Address {
        address: iota_address,
        balance: 0,
        key_index,
        internal: false,
        outputs: vec![],
    };
    Ok(address)
}

/// Gets an unused change address for the given account and address.
pub(crate) async fn get_new_change_address(
    account: &Account,
    address: &Address,
    metadata: GenerateAddressMetadata,
) -> crate::Result<Address> {
    let key_index = *address.key_index();
    let iota_address = get_iota_address(
        &account,
        key_index,
        true,
        address.address().bech32_hrp().to_string(),
        metadata,
    )
    .await?;
    let address = Address {
        address: iota_address,
        balance: 0,
        key_index,
        internal: true,
        outputs: vec![],
    };
    Ok(address)
}

pub(crate) fn is_unspent(account: &Account, address: &AddressWrapper) -> bool {
    !account
        .list_messages(0, 0, Some(MessageType::Sent))
        .iter()
        .any(|message| message.addresses().contains(&address.as_ref()))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn is_unspent_false() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        let address = crate::test_utils::generate_random_address();
        let spent_tx = crate::test_utils::GenerateMessageBuilder::default()
            .address(address.clone())
            .incoming(false)
            .build();

        account_handle.write().await.append_messages(vec![spent_tx]);

        let response = super::is_unspent(&*account_handle.read().await, address.address());
        assert_eq!(response, false);
    }

    #[tokio::test]
    async fn is_unspent_true() {
        let manager = crate::test_utils::get_account_manager().await;
        let account_handle = crate::test_utils::AccountCreator::new(&manager).create().await;
        let address = crate::test_utils::generate_random_iota_address();

        let response = super::is_unspent(&*account_handle.read().await, &address);
        assert_eq!(response, true);
    }
}
