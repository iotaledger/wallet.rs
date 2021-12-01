// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;

use iota_client::bee_message::address::Address;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

use std::{path::Path, sync::Arc};

#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
pub(crate) mod ledger;
#[cfg(feature = "mnemonic")]
pub(crate) mod mnemonic;
pub(crate) mod types;
pub use types::{GenerateAddressMetadata, LedgerStatus, Network, SignMessageMetadata, SignerType, TransactionInput};

type SignerHandle = Arc<Mutex<Box<dyn Signer + Sync + Send>>>;
type Signers = Arc<Mutex<SignerHandle>>;
static SIGNER_INSTANCE: OnceCell<Signers> = OnceCell::new();

/// Signer interface.
#[async_trait::async_trait]
pub trait Signer {
    /// Get the ledger status.
    async fn get_ledger_status(&self, is_simulator: bool) -> LedgerStatus;
    /// Initialises a mnemonic.
    async fn store_mnemonic(&mut self, storage_path: &Path, mnemonic: String) -> crate::Result<()>;
    /// Generates an address.
    async fn generate_address(
        &mut self,
        account: &Account,
        index: usize,
        internal: bool,
        metadata: GenerateAddressMetadata,
    ) -> crate::Result<Address>;
    /// Signs transaction essence.
    async fn sign_transaction<'a>(
        &mut self,
        account: &Account,
        essence: &iota_client::bee_message::prelude::Essence,
        inputs: &mut Vec<TransactionInput>,
        metadata: SignMessageMetadata<'a>,
    ) -> crate::Result<Vec<iota_client::bee_message::prelude::UnlockBlock>>;
}

fn default_signers() -> Signers {
    #[cfg(feature = "mnemonic")]
    let signer = self::mnemonic::MnemonicSigner::default();

    #[cfg(feature = "ledger-nano-simulator")]
    let signer = ledger::LedgerNanoSigner {
        is_simulator: true,
        ..Default::default()
    };

    #[cfg(feature = "ledger-nano")]
    let signer = ledger::LedgerNanoSigner {
        is_simulator: false,
        ..Default::default()
    };

    #[cfg(feature = "stronghold")]
    let signer = self::stronghold::StrongholdSigner::default();

    Arc::new(Mutex::new(Arc::new(Mutex::new(
        Box::new(signer) as Box<dyn Signer + Sync + Send>
    ))))
}

/// Sets the signer interface for the given type.
pub fn set_signer(signer_type: SignerType) {
    let signer = match signer_type {
        #[cfg(feature = "mnemonic")]
        SignerType::Mnemonic => Arc::new(Mutex::new(
            Box::new(self::mnemonic::MnemonicSigner::default()) as Box<dyn Signer + Sync + Send>
        )),
        #[cfg(feature = "ledger-nano")]
        // don't automatically consoldiate with ledger accounts, because they require approval from the user
        SignerType::LedgerNano => Arc::new(Mutex::new(Box::new(ledger::LedgerNanoSigner {
            is_simulator: false,
            ..Default::default()
        }) as Box<dyn Signer + Sync + Send>)),
        #[cfg(feature = "ledger-nano-simulator")]
        SignerType::LedgerNanoSimulator => Arc::new(Mutex::new(Box::new(ledger::LedgerNanoSimulator {
            is_simulator: true,
            ..Default::default()
        }) as Box<dyn Signer + Sync + Send>)),
        #[cfg(feature = "stronghold")]
        SignerType::Stronghold => Arc::new(Mutex::new(
            Box::new(self::stronghold::StrongholdSigner::default()) as Box<dyn Signer + Sync + Send>
        )),
    };
    let signer = Arc::new(Mutex::new(signer));
    SIGNER_INSTANCE.get_or_init(|| signer);
}

/// Gets the signer interface.
pub(crate) async fn get_signer() -> SignerHandle {
    SIGNER_INSTANCE.get_or_init(default_signers).lock().await.clone()
}
