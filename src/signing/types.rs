// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::types::address::AccountAddress;

use getset::Getters;
use iota_client::bee_message::input::Input;
use serde::{Deserialize, Serialize};

/// The signer types.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignerType {
    /// Stronghold signer.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    Stronghold,
    /// Ledger Device
    #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
    LedgerNano,
    /// Ledger Speculos Simulator
    #[cfg(feature = "ledger-nano-simulator")]
    LedgerNanoSimulator,
    /// Mnemonic, not recommended since it's not as secure as Stronghold or Ledger
    #[cfg(feature = "mnemonic")]
    Mnemonic,
}

/// Metadata provided to [sign_message](trait.Signer.html#method.sign_message).
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SignMessageMetadata<'a> {
    /// The transfer's remainder value.
    pub remainder_value: u64,
    /// The transfer's deposit address for the remainder value if any.
    pub remainder_deposit_address: Option<&'a AccountAddress>,
    /// The network which is used so the correct BIP32 path is used for the ledger. Debug mode starts with 44'/1' and
    /// in mainnet-mode it's 44'/4218'
    pub network: Network,
}

/// Metadata provided to [generate_address](trait.Signer.html#method.generate_address).
#[derive(Debug, Getters, Clone, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct GenerateAddressMetadata {
    /// Indicates that the address is being generated as part of the account syncing process.
    /// This means that the account might not be saved.
    /// If it is false, the prompt will be displayed on ledger devices.
    pub syncing: bool,
    /// The network which is used so the correct BIP32 path is used for the ledger. Debug mode starts with 44'/1' and
    /// in mainnet-mode it's 44'/4218'
    pub network: Network,
}

/// Network enum for ledger metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Network {
    /// Mainnet
    Mainnet,
    /// Testnet
    Testnet,
}

/// The Ledger device status.
#[derive(Debug, ::serde::Serialize)]
pub struct LedgerApp {
    /// Opened app name.
    name: String,
    /// Opened app version.
    version: String,
}

/// The Ledger device status.
#[derive(Debug, ::serde::Serialize)]
pub struct LedgerStatus {
    /// Ledger is available and ready to be used.
    pub(crate) connected: bool,
    /// Ledger is connected and locked.
    pub(crate) locked: bool,
    /// Ledger opened app.
    pub(crate) app: Option<LedgerApp>,
}

/// One of the transaction inputs and its address information needed for signing it.
pub struct TransactionInput {
    /// The input.
    pub input: Input,
    /// Input's address index.
    pub address_index: usize,
    /// Whether the input address is a change address or a public address.
    pub address_internal: bool,
}
