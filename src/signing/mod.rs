// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    account::Account,
    address::{Address, IotaAddress},
};
use crypto::keys::bip39::Mnemonic;
use getset::Getters;
use iota_client::bee_message::input::Input;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
pub(crate) mod ledger;

#[cfg(feature = "stronghold")]
pub(crate) mod stronghold;

type SignerHandle = Arc<Mutex<Box<dyn Signer + Sync + Send>>>;
type Signers = Arc<Mutex<HashMap<SignerType, SignerHandle>>>;
static SIGNERS_INSTANCE: OnceCell<Signers> = OnceCell::new();

/// The signer types.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SignerType {
    /// Stronghold signer.
    #[cfg(feature = "stronghold")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
    Stronghold,
    /// Ledger Device
    #[cfg(feature = "ledger-nano")]
    LedgerNano,
    /// Ledger Speculos Simulator
    #[cfg(feature = "ledger-nano-simulator")]
    LedgerNanoSimulator,
    /// Custom signer with its identifier.
    Custom(String),
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

/// Network enum for ledger metadata
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Network {
    /// Mainnet
    Mainnet,
    /// Testnet
    Testnet,
}

/// Metadata provided to [generate_address](trait.Signer.html#method.generate_address).
#[derive(Getters, Clone)]
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

/// Metadata provided to [sign_message](trait.Signer.html#method.sign_message).
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SignMessageMetadata<'a> {
    /// The transfer's address that has remainder value if any.
    pub remainder_address: Option<&'a Address>,
    /// The transfer's remainder value.
    pub remainder_value: u64,
    /// The transfer's deposit address for the remainder value if any.
    pub remainder_deposit_address: Option<&'a Address>,
    /// The network which is used so the correct BIP32 path is used for the ledger. Debug mode starts with 44'/1' and
    /// in mainnet-mode it's 44'/4218'
    pub network: Network,
}

/// Signer interface.
#[async_trait::async_trait]
pub trait Signer {
    /// Get the ledger status.
    async fn get_ledger_status(&self, is_simulator: bool) -> crate::LedgerStatus;
    /// Initialises a mnemonic.
    async fn store_mnemonic(&mut self, storage_path: &Path, mnemonic: Mnemonic) -> crate::Result<()>;
    /// Generates an address.
    async fn generate_address(
        &mut self,
        account: &Account,
        index: usize,
        internal: bool,
        metadata: GenerateAddressMetadata,
    ) -> crate::Result<IotaAddress>;
    /// Signs message.
    async fn sign_message<'a>(
        &mut self,
        account: &Account,
        essence: &iota_client::bee_message::prelude::Essence,
        inputs: &mut Vec<TransactionInput>,
        metadata: SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota_client::bee_message::prelude::UnlockBlock>>;
}

fn default_signers() -> Signers {
    let mut signers = HashMap::new();

    #[cfg(feature = "stronghold")]
    {
        signers.insert(
            SignerType::Stronghold,
            Arc::new(Mutex::new(
                Box::<self::stronghold::StrongholdSigner>::default() as Box<dyn Signer + Sync + Send>
            )),
        );
    }

    #[cfg(feature = "ledger-nano")]
    {
        signers.insert(
            SignerType::LedgerNano,
            Arc::new(Mutex::new(Box::new(ledger::LedgerNanoSigner {
                is_simulator: false,
                ..Default::default()
            }) as Box<dyn Signer + Sync + Send>)),
        );
    }

    #[cfg(feature = "ledger-nano-simulator")]
    {
        signers.insert(
            SignerType::LedgerNanoSimulator,
            Arc::new(Mutex::new(Box::new(ledger::LedgerNanoSigner {
                is_simulator: true,
                ..Default::default()
            }) as Box<dyn Signer + Sync + Send>)),
        );
    }

    Arc::new(Mutex::new(signers))
}

/// Sets the signer interface for the given type.
pub async fn set_signer<S: Signer + Sync + Send + 'static>(signer_type: SignerType, signer: S) {
    SIGNERS_INSTANCE
        .get_or_init(default_signers)
        .lock()
        .await
        .insert(signer_type, Arc::new(Mutex::new(Box::new(signer))));
}

/// Gets the signer interface.
pub(crate) async fn get_signer(signer_type: &SignerType) -> SignerHandle {
    SIGNERS_INSTANCE
        .get_or_init(default_signers)
        .lock()
        .await
        .get(signer_type)
        .cloned()
        .unwrap_or_else(|| panic!("signer not initialized for type {:?}", signer_type))
}
