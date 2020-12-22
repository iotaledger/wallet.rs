// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Arc};

use crate::account::Account;
use iota::Input;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use slip10::BIP32Path;
use tokio::sync::Mutex;

mod env_mnemonic;
#[cfg(feature = "stronghold")]
mod stronghold;
use env_mnemonic::EnvMnemonicSigner;

type SignerHandle = Arc<Mutex<Box<dyn Signer + Sync + Send>>>;
type Signers = Arc<Mutex<HashMap<SignerType, SignerHandle>>>;
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
    /// Whether the input address is a change address or a public address.
    pub address_internal: bool,
}

/// Signer interface.
#[async_trait::async_trait]
pub trait Signer {
    /// Initialises an account.
    async fn init_account(&self, account: &Account, mnemonic: Option<String>) -> crate::Result<String>;
    /// Generates an address.
    async fn generate_address(&self, account: &Account, index: usize, internal: bool) -> crate::Result<iota::Address>;
    /// Signs message.
    async fn sign_message(
        &self,
        account: &Account,
        essence: &iota::TransactionEssence,
        inputs: &mut Vec<TransactionInput>,
    ) -> crate::Result<Vec<iota::UnlockBlock>>;
}

fn default_signers() -> Signers {
    let mut signers = HashMap::new();

    #[cfg(feature = "stronghold")]
    {
        signers.insert(
            SignerType::Stronghold,
            Arc::new(Mutex::new(
                Box::new(self::stronghold::StrongholdSigner::default()) as Box<dyn Signer + Sync + Send>
            )),
        );
    }

    signers.insert(
        SignerType::EnvMnemonic,
        Arc::new(Mutex::new(
            Box::new(EnvMnemonicSigner::default()) as Box<dyn Signer + Sync + Send>
        )),
    );

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
        .expect(&format!("signer not initialized for type {:?}", signer_type))
}
