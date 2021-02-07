// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
#[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
pub(crate) mod stronghold;

pub use error::Error;

#[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
pub use stronghold::{
    get_status as get_stronghold_status, set_password_clear_interval as set_stronghold_password_clear_interval,
    unload_snapshot as lock_stronghold, SnapshotStatus as StrongholdSnapshotStatus, Status as StrongholdStatus,
};

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
#[cfg_attr(docsrs, doc(cfg(any(feature = "stronghold", feature = "stronghold-storage"))))]
pub async fn with_actor_system<F: FnOnce(&riker::actors::ActorSystem)>(cb: F) {
    let runtime = self::stronghold::actor_runtime().lock().await;
    cb(&runtime.stronghold.system)
}

/// Opens the IOTA app on Ledger (Nano S/X or Speculos simulator).
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
pub fn open_ledger_app(is_simulator: bool) -> crate::Result<()> {
    iota_ledger::get_ledger(signing::ledger::HARDENED, is_simulator)?;
    Ok(())
}

#[cfg(test)]
mod test_utils {
    use super::{
        account::AccountHandle,
        account_manager::{AccountManager, ManagerStorage},
        address::{Address, AddressBuilder, AddressWrapper},
        client::ClientOptionsBuilder,
        message::Message,
        signing::SignerType,
    };
    use iota::{
        pow::providers::{Provider as PowProvider, ProviderBuilder as PowProviderBuilder},
        Address as IotaAddress, Ed25519Address, Ed25519Signature, MessageId, Payload, SignatureLockedSingleOutput,
        SignatureUnlock, TransactionId, TransactionPayloadBuilder, TransactionPayloadEssence, UTXOInput, UnlockBlock,
    };
    use once_cell::sync::OnceCell;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{atomic::AtomicBool, Arc},
    };
    use tokio::sync::Mutex;

    type GeneratedAddressMap = HashMap<(String, usize, bool), iota::Ed25519Address>;
    static TEST_SIGNER_GENERATED_ADDRESSES: OnceCell<Mutex<GeneratedAddressMap>> = OnceCell::new();

    #[derive(Default)]
    struct TestSigner;

    #[async_trait::async_trait]
    impl crate::signing::Signer for TestSigner {
        async fn store_mnemonic(&mut self, _: &PathBuf, _mnemonic: String) -> crate::Result<()> {
            Ok(())
        }

        async fn generate_address(
            &mut self,
            account: &crate::account::Account,
            address_index: usize,
            internal: bool,
            _metadata: crate::signing::GenerateAddressMetadata,
        ) -> crate::Result<iota::Address> {
            // store and read the generated addresses from the static map so the generation is deterministic
            let generated_addresses = TEST_SIGNER_GENERATED_ADDRESSES.get_or_init(Default::default);
            let mut generated_addresses = generated_addresses.lock().await;
            let key = (account.id().clone(), address_index, internal);
            if let Some(address) = generated_addresses.get(&key) {
                Ok(iota::Address::Ed25519(*address))
            } else {
                let mut address = [0; iota::ED25519_ADDRESS_LENGTH];
                crypto::rand::fill(&mut address).unwrap();
                let address = iota::Ed25519Address::new(address);
                generated_addresses.insert(key, address);
                Ok(iota::Address::Ed25519(address))
            }
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

    #[derive(Default)]
    struct TestStorage {
        cache: HashMap<String, String>,
    }

    #[async_trait::async_trait]
    impl crate::storage::StorageAdapter for TestStorage {
        async fn get(&mut self, account_id: &str) -> crate::Result<String> {
            match self.cache.get(account_id) {
                Some(value) => Ok(value.to_string()),
                None => Err(crate::Error::AccountNotFound),
            }
        }

        async fn get_all(&mut self) -> crate::Result<Vec<String>> {
            Ok(self.cache.values().cloned().collect())
        }

        async fn set(&mut self, account_id: &str, account: String) -> crate::Result<()> {
            self.cache.insert(account_id.to_string(), account);
            Ok(())
        }

        async fn remove(&mut self, account_id: &str) -> crate::Result<()> {
            self.cache.remove(account_id).ok_or(crate::Error::AccountNotFound)?;
            Ok(())
        }
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
            .skip_polling()
            .finish()
            .await
            .unwrap();

        let signer_type = SignerType::Custom("".to_string());
        crate::signing::set_signer(signer_type.clone(), TestSigner::default()).await;
        manager.store_mnemonic(signer_type, None).await.unwrap();

        #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
        manager.set_stronghold_password("password").await.unwrap();

        #[cfg(feature = "stronghold")]
        manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

        manager
    }

    struct StorageTestCase {
        storage_password: Option<String>,
        storage: ManagerStorage,
    }

    enum ManagerTestCase {
        Signer(SignerType),
        Storage(StorageTestCase),
    }

    #[derive(PartialEq)]
    pub enum TestType {
        Signing,
        Storage,
        SigningAndStorage,
    }

    pub async fn with_account_manager<R: futures::Future<Output = ()>, F: Fn(AccountManager, SignerType) -> R>(
        test_type: TestType,
        cb: F,
    ) {
        let mut test_cases: Vec<ManagerTestCase> = Vec::new();

        if test_type == TestType::Signing || test_type == TestType::SigningAndStorage {
            test_cases.push(ManagerTestCase::Signer(SignerType::Custom("".to_string())));
            #[cfg(feature = "stronghold")]
            {
                test_cases.push(ManagerTestCase::Signer(SignerType::Stronghold));
            }
        }

        if test_type == TestType::Storage || test_type == TestType::SigningAndStorage {
            // ---- Stronghold storage ----
            #[cfg(feature = "stronghold-storage")]
            {
                test_cases.push(ManagerTestCase::Storage(StorageTestCase {
                    storage_password: None,
                    storage: ManagerStorage::Stronghold,
                }));

                test_cases.push(ManagerTestCase::Storage(StorageTestCase {
                    storage_password: Some("password".to_string()),
                    storage: ManagerStorage::Stronghold,
                }));
            }

            // ---- SQLite storage ----
            #[cfg(feature = "sqlite-storage")]
            {
                test_cases.push(ManagerTestCase::Storage(StorageTestCase {
                    storage_password: None,
                    storage: ManagerStorage::Sqlite,
                }));

                test_cases.push(ManagerTestCase::Storage(StorageTestCase {
                    storage_password: Some("password".to_string()),
                    storage: ManagerStorage::Sqlite,
                }));
            }
        }

        for test_case in test_cases {
            let storage_path = loop {
                let storage_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
                let storage_path = PathBuf::from(format!("./test-storage/{}", storage_path));
                if !storage_path.exists() {
                    std::fs::create_dir_all(&storage_path).unwrap();
                    break storage_path;
                }
            };

            let mut manager_builder = AccountManager::builder();

            let signer_type = match test_case {
                ManagerTestCase::Signer(signer_type) => {
                    crate::signing::set_signer(signer_type.clone(), TestSigner::default()).await;
                    manager_builder = manager_builder
                        .with_storage(
                            storage_path,
                            ManagerStorage::Custom(Box::new(TestStorage::default())),
                            None,
                        )
                        .unwrap();
                    signer_type
                }
                ManagerTestCase::Storage(config) => {
                    manager_builder = manager_builder
                        .with_storage(storage_path, config.storage, config.storage_password.as_deref())
                        .unwrap();
                    #[cfg(feature = "stronghold")]
                    let signer_type = SignerType::Stronghold;
                    #[cfg(not(feature = "stronghold"))]
                    let signer_type = {
                        let signer_type = SignerType::Custom("".to_string());
                        crate::signing::set_signer(signer_type.clone(), TestSigner::default()).await;
                        signer_type
                    };
                    signer_type
                }
            };

            let mut manager = manager_builder
                .skip_polling()
                .finish()
                .await
                .unwrap();

            #[cfg(any(feature = "stronghold", feature = "stronghold-storage"))]
            manager.set_stronghold_password("password").await.unwrap();

            manager.store_mnemonic(signer_type.clone(), None).await.unwrap();

            cb(manager, signer_type).await;
        }
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

        fn nonce(
            &self,
            _bytes: &[u8],
            _target_score: f64,
            _done: Option<Arc<AtomicBool>>,
        ) -> std::result::Result<u64, Self::Error> {
            Ok(0)
        }
    }

    pub struct AccountCreator<'a> {
        manager: &'a AccountManager,
        addresses: Vec<Address>,
        messages: Vec<Message>,
        signer_type: Option<SignerType>,
    }

    impl<'a> AccountCreator<'a> {
        pub fn new(manager: &'a AccountManager) -> Self {
            Self {
                manager,
                addresses: Vec::new(),
                messages: Vec::new(),
                signer_type: None,
            }
        }

        pub fn addresses(mut self, addresses: Vec<Address>) -> Self {
            self.addresses = addresses;
            self
        }

        pub fn messages(mut self, messages: Vec<Message>) -> Self {
            self.messages = messages;
            self
        }

        pub fn signer_type(mut self, signer_type: SignerType) -> Self {
            self.signer_type.replace(signer_type);
            self
        }

        pub async fn create(self) -> AccountHandle {
            let client_options = ClientOptionsBuilder::new()
                .with_node("https://api.lb-0.testnet.chrysalis2.com")
                .expect("invalid node URL")
                .build()
                .unwrap();

            let mut account_initialiser = self.manager.create_account(client_options).unwrap();
            if let Some(signer_type) = self.signer_type {
                account_initialiser = account_initialiser.signer_type(signer_type);
            }
            account_initialiser
                .alias("alias")
                .messages(self.messages)
                .addresses(self.addresses)
                .initialise()
                .await
                .expect("failed to add account")
        }
    }

    pub fn generate_random_iota_address() -> AddressWrapper {
        AddressWrapper::new(
            IotaAddress::Ed25519(Ed25519Address::new(rand::random::<[u8; 32]>())),
            "iota".to_string(),
        )
    }

    pub fn generate_random_address() -> Address {
        AddressBuilder::new()
            .key_index(0)
            .address(generate_random_iota_address())
            .balance(0)
            .outputs(Vec::new())
            .build()
            .unwrap()
    }

    macro_rules! builder_setters {
        ($ty:ident, $($x:ident => $type:ty),*) => {
            impl $ty {
                $(
                    pub fn $x(mut self, value: $type) -> Self {
                        self.$x = value;
                        self
                    }
                )*
            }
        }
    }

    pub struct GenerateMessageBuilder {
        value: u64,
        address: Address,
        confirmed: Option<bool>,
        broadcasted: bool,
        incoming: bool,
        input_transaction_id: TransactionId,
    }

    impl Default for GenerateMessageBuilder {
        fn default() -> Self {
            Self {
                value: rand::thread_rng().gen_range(1, 50000),
                address: generate_random_address(),
                confirmed: Some(false),
                broadcasted: false,
                incoming: false,
                input_transaction_id: TransactionId::new([0; 32]),
            }
        }
    }

    builder_setters!(
        GenerateMessageBuilder,
        value => u64,
        address => Address,
        confirmed => Option<bool>,
        broadcasted => bool,
        incoming => bool,
        input_transaction_id => TransactionId
    );

    impl GenerateMessageBuilder {
        pub fn build(self) -> Message {
            Message {
                id: MessageId::new([0; 32]),
                version: 1,
                parents: vec![MessageId::new([0; 32])],
                payload_length: 0,
                payload: Payload::Transaction(Box::new(
                    TransactionPayloadBuilder::new()
                        .with_essence(
                            TransactionPayloadEssence::builder()
                                .add_output(
                                    SignatureLockedSingleOutput::new(*self.address.address().as_ref(), self.value)
                                        .unwrap()
                                        .into(),
                                )
                                .add_input(UTXOInput::new(self.input_transaction_id, 0).unwrap().into())
                                .finish()
                                .unwrap(),
                        )
                        .add_unlock_block(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                            [0; 32],
                            Box::new([0]),
                        ))))
                        .finish()
                        .unwrap(),
                )),
                timestamp: chrono::Utc::now(),
                nonce: 0,
                value: self.value,
                remainder_value: 0,
                confirmed: self.confirmed,
                broadcasted: self.broadcasted,
                incoming: self.incoming,
            }
        }
    }
}
