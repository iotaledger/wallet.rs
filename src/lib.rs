// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]

/// The account module.
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The actor interface for the library.
pub mod actor;
/// The address module.
pub mod address;
/// The client module.
pub mod client;
pub(crate) mod error;
/// The event module.
pub mod event;
/// The message module.
pub mod message;
/// The monitor module.
pub mod monitor;
pub(crate) mod serde;
/// Signing interfaces.
pub mod signing;
/// The storage module.
pub mod storage;
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
pub(crate) mod stronghold;

pub use error::Error;

pub use stronghold::set_password_clear_interval as set_stronghold_password_clear_interval;

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
pub use chrono::prelude::{DateTime, Local, Utc};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Mutex<Runtime>> = OnceCell::new();

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().block_on(cb)
}

pub(crate) fn spawn<F>(future: F)
where
    F: futures::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let runtime = RUNTIME.get_or_init(|| Mutex::new(Runtime::new().unwrap()));
    runtime.lock().unwrap().spawn(future);
}

/// Access the stronghold's actor system.
#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
pub async fn with_actor_system<F: FnOnce(&riker::actors::ActorSystem)>(cb: F) {
    let runtime = self::stronghold::actor_runtime().lock().await;
    cb(&runtime.stronghold.system)
}

#[cfg(test)]
mod test_utils {
    use super::{
        account_manager::{AccountManager, ManagerStorage},
        signing::SignerType,
    };
    use iota::pow::providers::{Provider as PowProvider, ProviderBuilder as PowProviderBuilder};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::{path::PathBuf, time::Duration};

    static POLLING_INTERVAL: Duration = Duration::from_secs(2);

    struct TestSigner {}

    #[async_trait::async_trait]
    impl crate::signing::Signer for TestSigner {
        async fn store_mnemonic(&mut self, _: &PathBuf, _mnemonic: String) -> crate::Result<()> {
            Ok(())
        }

        async fn generate_address(
            &mut self,
            _account: &crate::account::Account,
            _address_index: usize,
            _internal: bool,
            _metadata: crate::signing::GenerateAddressMetadata,
        ) -> crate::Result<iota::Address> {
            let mut address = [0; iota::ED25519_ADDRESS_LENGTH];
            crypto::rand::fill(&mut address).unwrap();
            Ok(iota::Address::Ed25519(iota::Ed25519Address::new(address)))
        }

        async fn sign_message<'a>(
            &mut self,
            _account: &crate::account::Account,
            _essence: &iota::TransactionPayloadEssence,
            _inputs: &mut Vec<crate::signing::TransactionInput>,
            _metadata: crate::signing::SignMessageMetadata<'a>,
        ) -> crate::Result<Vec<iota::UnlockBlock>> {
            Ok(Vec::new())
        }
    }

    pub fn signer_type() -> SignerType {
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        let signer_type = SignerType::Stronghold;
        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        let signer_type = SignerType::Custom("".to_string());
        signer_type
    }

    pub async fn get_account_manager() -> AccountManager {
        let storage_path = loop {
            let storage_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
            let storage_path = PathBuf::from(format!("./test-storage/{}", storage_path));
            if !storage_path.exists() {
                break storage_path;
            }
        };

        #[cfg(all(feature = "stronghold-storage", feature = "sqlite-storage"))]
        let default_storage = ManagerStorage::Stronghold;
        #[cfg(all(feature = "stronghold-storage", not(feature = "sqlite-storage")))]
        let default_storage = ManagerStorage::Stronghold;
        #[cfg(all(feature = "sqlite-storage", not(feature = "stronghold-storage")))]
        let default_storage = ManagerStorage::Sqlite;

        let mut manager = AccountManager::builder()
            .with_storage(storage_path, default_storage, Some("password"))
            .unwrap()
            .with_polling_interval(POLLING_INTERVAL)
            .finish()
            .await
            .unwrap();

        #[cfg(not(any(feature = "stronghold", feature = "stronghold-storage")))]
        crate::signing::set_signer(signer_type(), TestSigner {}).await;
        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        manager.set_stronghold_password("password").await.unwrap();

        manager.store_mnemonic(signer_type(), None).await.unwrap();

        manager
    }

    /// The miner builder.
    #[derive(Default)]
    pub struct NoopNonceProviderBuilder;

    impl PowProviderBuilder for NoopNonceProviderBuilder {
        type Provider = NoopNonceProvider;

        fn new() -> Self {
            Self::default()
        }

        fn finish(self) -> NoopNonceProvider {
            NoopNonceProvider {}
        }
    }

    /// The miner used for PoW
    pub struct NoopNonceProvider;

    impl PowProvider for NoopNonceProvider {
        type Builder = NoopNonceProviderBuilder;
        type Error = crate::Error;

        fn nonce(&self, _bytes: &[u8], _target_score: f64) -> std::result::Result<u64, Self::Error> {
            Ok(0)
        }
    }
}
