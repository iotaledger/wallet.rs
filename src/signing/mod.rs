// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    account::Account,
    address::{Address, IotaAddress},
};
use getset::Getters;
use iota::Input;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use slip10::BIP32Path;
use tokio::sync::Mutex;

#[cfg(feature = "stronghold")]
mod stronghold;

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
pub struct SignMessageMetadata<'a> {
    /// The transfer's address that has remainder value if any.
    pub(crate) remainder_address: Option<&'a Address>,
    /// The transfer's remainder value.
    pub(crate) remainder_value: u64,
    /// The transfer's deposit address for the remainder value if any.
    pub(crate) remainder_deposit_address: Option<&'a Address>,
}

/// Signer interface.
#[async_trait::async_trait]
pub trait Signer {
    /// Initialises a mnemonic.
    async fn store_mnemonic(&self, storage_path: &PathBuf, mnemonic: String) -> crate::Result<()>;
    /// Generates an address.
    async fn generate_address(
        &self,
        account: &Account,
        index: usize,
        internal: bool,
        metadata: GenerateAddressMetadata,
    ) -> crate::Result<IotaAddress>;
    /// Signs message.
    async fn sign_message<'a>(
        &self,
        account: &Account,
        essence: &iota::TransactionPayloadEssence,
        inputs: &mut Vec<TransactionInput>,
        metadata: SignMessageMetadata<'a>,
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
