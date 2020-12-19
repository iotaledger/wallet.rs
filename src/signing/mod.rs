// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{account::Account, address::IotaAddress};
use getset::Getters;
use iota::Input;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use slip10::BIP32Path;

mod stronghold;
use self::stronghold::StrongholdSigner;
mod env_mnemonic;
use env_mnemonic::EnvMnemonicSigner;

type BoxedSigner = Box<dyn Signer + Sync + Send>;
type Signers = Arc<RwLock<HashMap<SignerType, BoxedSigner>>>;
static SIGNERS_INSTANCE: OnceCell<Signers> = OnceCell::new();

/// The signer types.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SignerType {
    /// Stronghold signer.
    #[cfg(feature = "stronghold")]
    Stronghold,
    /// Mnemonic through environment variable.
    EnvMnemonic,
    /// Custom signer with its identifier.
    Custom(String),
}

/// One of the transaction inputs and its address information needed for signing it.
pub struct TransactionInput {
    /// The input.
    pub input: Input,
    /// Input's address index.
    pub address_index: usize,
    /// Input's address BIP32 derivation path.
    pub address_path: BIP32Path,
}

/// Metadata provided to [generate_address](trait.Signer.html#method.generate_address).
#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct GenerateAddressMetadata {
    /// Indicates that the address is being generated as part of the account syncing process.
    /// This means that the account might not be saved.
    pub(crate) syncing: bool,
}

/// Metadata provided to [sign_message](trait.Signer.html#method.sign_message).
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SignMessageMetadata {
    /// The transfer's address that has remainder value if any.
    pub(crate) remainder_address: Option<IotaAddress>,
    /// The transfer's remainder value.
    pub(crate) remainder_value: u64,
    /// The transfer's deposit address for the remainder value if any.
    pub(crate) remainder_deposit_address: Option<IotaAddress>,
}

/// Signer interface.
pub trait Signer {
    /// Initialises an account.
    fn init_account(&self, account: &Account, mnemonic: Option<String>) -> crate::Result<String>;
    /// Generates an address.
    fn generate_address(
        &self,
        account: &Account,
        index: usize,
        internal: bool,
        metadata: GenerateAddressMetadata,
    ) -> crate::Result<IotaAddress>;
    /// Signs message.
    fn sign_message(
        &self,
        account: &Account,
        essence: &iota::TransactionEssence,
        inputs: &mut Vec<TransactionInput>,
        metadata: SignMessageMetadata,
    ) -> crate::Result<Vec<iota::UnlockBlock>>;
}

fn default_signers() -> Signers {
    let mut signers = HashMap::new();

    #[cfg(feature = "stronghold")]
    {
        signers.insert(
            SignerType::Stronghold,
            Box::new(StrongholdSigner::default()) as Box<dyn Signer + Sync + Send>,
        );
    }

    signers.insert(
        SignerType::EnvMnemonic,
        Box::new(EnvMnemonicSigner::default()) as Box<dyn Signer + Sync + Send>,
    );

    Arc::new(RwLock::new(signers))
}

/// Sets the signer interface for the given type.
pub fn set_signer<S: Signer + Sync + Send + 'static>(signer_type: SignerType, signer: S) {
    let mut instances = SIGNERS_INSTANCE.get_or_init(default_signers).write().unwrap();
    instances.insert(signer_type, Box::new(signer));
}

/// Gets the signer interface.
pub(crate) fn with_signer<T, F: FnOnce(&BoxedSigner) -> T>(signer_type: &SignerType, cb: F) -> T {
    let instances = SIGNERS_INSTANCE.get_or_init(default_signers).read().unwrap();
    if let Some(instance) = instances.get(signer_type) {
        cb(instance)
    } else {
        panic!(format!("signer not initialized for type {:?}", signer_type))
    }
}
