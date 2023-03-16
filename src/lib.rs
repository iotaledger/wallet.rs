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
#[cfg(feature = "participation")]
/// Participation interfaces.
pub mod participation;
pub(crate) mod serde;
/// Signing interfaces.
pub mod signing;
/// The storage module.
pub(crate) mod storage;
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub(crate) mod stronghold;

pub use error::Error;

pub use storage::remove as remove_storage;
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub use stronghold::{
    get_status as get_stronghold_status, set_password_clear_interval as set_stronghold_password_clear_interval,
    unload_snapshot as lock_stronghold, SnapshotStatus as StrongholdSnapshotStatus, Status as StrongholdStatus,
};

/// The wallet Result type.
pub type Result<T> = std::result::Result<T, Error>;
pub use chrono::prelude::{DateTime, Local, Utc};
pub use iota_client;
pub use iota_migration;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub(crate) fn block_on<C: futures::Future>(cb: C) -> C::Output {
    let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
    runtime.block_on(cb)
}

pub(crate) fn spawn<F>(future: F)
where
    F: futures::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
    runtime.spawn(future);
}

/// Access the stronghold's actor system.
#[cfg(feature = "stronghold")]
#[cfg_attr(docsrs, doc(cfg(feature = "stronghold")))]
pub async fn with_actor_system<F: FnOnce(&riker::actors::ActorSystem)>(cb: F) {
    let runtime = self::stronghold::actor_runtime().lock().await;
    cb(&runtime.stronghold.system)
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
    connected: bool,
    /// Ledger is connected and locked.
    locked: bool,
    /// Ledger opened app.
    app: Option<LedgerApp>,
}

/// Gets the status of the Ledger device/simulator.
#[allow(unreachable_code)]
#[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))))]
pub async fn get_ledger_status(is_simulator: bool) -> LedgerStatus {
    if is_simulator {
        #[cfg(feature = "ledger-nano-simulator")]
        {
            let simulator_signer = crate::signing::get_signer(&crate::signing::SignerType::LedgerNanoSimulator).await;
            let signer = simulator_signer.lock().await;
            return signer.get_ledger_status(is_simulator).await;
        }
    } else {
        #[cfg(feature = "ledger-nano")]
        {
            let ledger_signer = crate::signing::get_signer(&crate::signing::SignerType::LedgerNano).await;
            let signer = ledger_signer.lock().await;
            return signer.get_ledger_status(is_simulator).await;
        }
    }
    // dummy response
    LedgerStatus {
        connected: false,
        locked: false,
        app: None,
    }
}

#[cfg(test)]
mod test_utils {
    use super::{
        account::AccountHandle,
        account_manager::{AccountManager, AccountStore},
        address::{Address, AddressBuilder, AddressWrapper},
        client::ClientOptionsBuilder,
        message::{Message, MessagePayload, TransactionBuilderMetadata, TransactionEssence},
        signing::SignerType,
    };
    use iota_client::{
        bee_message::prelude::{
            Address as IotaAddress, Ed25519Address, Ed25519Signature, Essence, MessageId, Payload,
            SignatureLockedSingleOutput, SignatureUnlock, TransactionId, TransactionPayloadBuilder, UnlockBlock,
            UnlockBlocks, UtxoInput,
        },
        pow::providers::{NonceProvider, NonceProviderBuilder},
    };
    use once_cell::sync::OnceCell;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };
    use tokio::sync::Mutex;

    type GeneratedAddressMap = HashMap<(String, usize, bool), iota_client::bee_message::address::Ed25519Address>;
    static TEST_SIGNER_GENERATED_ADDRESSES: OnceCell<Mutex<GeneratedAddressMap>> = OnceCell::new();

    #[derive(Default)]
    struct TestSigner;

    #[async_trait::async_trait]
    impl crate::signing::Signer for TestSigner {
        async fn get_ledger_status(&self, _is_simulator: bool) -> crate::LedgerStatus {
            // dummy status
            crate::LedgerStatus {
                connected: false,
                locked: false,
                app: None,
            }
        }

        async fn store_mnemonic(&mut self, _: &Path, _mnemonic: String) -> crate::Result<()> {
            Ok(())
        }

        async fn generate_address(
            &mut self,
            account: &crate::account::Account,
            address_index: usize,
            internal: bool,
            _metadata: crate::signing::GenerateAddressMetadata,
        ) -> crate::Result<iota_client::bee_message::address::Address> {
            // store and read the generated addresses from the static map so the generation is deterministic
            let generated_addresses = TEST_SIGNER_GENERATED_ADDRESSES.get_or_init(Default::default);
            let mut generated_addresses = generated_addresses.lock().await;
            let key = (account.id().clone(), address_index, internal);
            if let Some(address) = generated_addresses.get(&key) {
                Ok(iota_client::bee_message::address::Address::Ed25519(*address))
            } else {
                let mut address = [0; iota_client::bee_message::address::ED25519_ADDRESS_LENGTH];
                crypto::utils::rand::fill(&mut address).unwrap();
                let address = iota_client::bee_message::address::Ed25519Address::new(address);
                generated_addresses.insert(key, address);
                Ok(iota_client::bee_message::address::Address::Ed25519(address))
            }
        }

        async fn sign_message<'a>(
            &mut self,
            _account: &crate::account::Account,
            _essence: &iota_client::bee_message::prelude::Essence,
            _inputs: &mut Vec<crate::signing::TransactionInput>,
            _metadata: crate::signing::SignMessageMetadata<'a>,
        ) -> crate::Result<Vec<iota_client::bee_message::prelude::UnlockBlock>> {
            Ok(Vec::new())
        }
    }

    #[derive(Default)]
    struct TestStorage {
        cache: HashMap<String, String>,
    }

    #[async_trait::async_trait]
    impl crate::storage::StorageAdapter for TestStorage {
        async fn get(&self, id: &str) -> crate::Result<String> {
            match self.cache.get(id) {
                Some(value) => Ok(value.to_string()),
                None => Err(crate::Error::RecordNotFound),
            }
        }

        async fn set(&mut self, id: &str, record: String) -> crate::Result<()> {
            self.cache.insert(id.to_string(), record);
            Ok(())
        }

        async fn batch_set(&mut self, records: HashMap<String, String>) -> crate::Result<()> {
            for (id, record) in records {
                self.cache.insert(id, record);
            }
            Ok(())
        }

        async fn remove(&mut self, id: &str) -> crate::Result<()> {
            self.cache.remove(id).ok_or(crate::Error::RecordNotFound)?;
            Ok(())
        }
    }

    pub async fn get_account_manager() -> AccountManager {
        let storage_path = loop {
            let storage_path: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(10)
                .collect();
            let storage_path = PathBuf::from(format!("./test-storage/{}", storage_path));
            if !storage_path.exists() {
                break storage_path;
            }
        };

        let manager = AccountManager::builder()
            .with_storage(storage_path, None)
            .unwrap()
            .with_skip_polling()
            .finish()
            .await
            .unwrap();

        let signer_type = SignerType::Custom("".to_string());
        crate::signing::set_signer(signer_type.clone(), TestSigner::default()).await;
        manager.store_mnemonic(signer_type, None).await.unwrap();

        #[cfg(feature = "stronghold")]
        manager.set_stronghold_password("password").await.unwrap();

        #[cfg(feature = "stronghold")]
        manager.store_mnemonic(SignerType::Stronghold, None).await.unwrap();

        manager
    }

    struct StorageTestCase {
        storage_password: Option<String>,
    }

    enum ManagerTestCase {
        Signer(SignerType),
        Storage(StorageTestCase),
    }

    #[derive(Eq, PartialEq)]
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
            test_cases.push(ManagerTestCase::Storage(StorageTestCase { storage_password: None }));
            test_cases.push(ManagerTestCase::Storage(StorageTestCase {
                storage_password: Some("password".to_string()),
            }));
        }

        for test_case in test_cases {
            let storage_path = loop {
                let storage_path: String = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .map(char::from)
                    .take(10)
                    .collect();
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
                    manager_builder = manager_builder.with_storage(storage_path, None).unwrap();
                    signer_type
                }
                ManagerTestCase::Storage(config) => {
                    manager_builder = manager_builder
                        .with_storage(storage_path, config.storage_password.as_deref())
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

            let manager = manager_builder.with_skip_polling().finish().await.unwrap();

            #[cfg(feature = "stronghold")]
            manager.set_stronghold_password("password").await.unwrap();

            manager.store_mnemonic(signer_type.clone(), None).await.unwrap();

            cb(manager, signer_type).await;
        }
    }

    /// The miner builder.
    #[derive(Default)]
    pub struct NoopNonceProviderBuilder;

    impl NonceProviderBuilder for NoopNonceProviderBuilder {
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

    impl NonceProvider for NoopNonceProvider {
        type Builder = NoopNonceProviderBuilder;
        type Error = crate::Error;

        fn nonce(&self, _bytes: &[u8], _target_score: f64) -> std::result::Result<u64, Self::Error> {
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
                .with_node("https://api.lb-0.h.chrysalis-devnet.iota.cafe")
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
            "atoi".to_string(),
        )
    }

    pub fn generate_random_address() -> Address {
        AddressBuilder::new()
            .key_index(0)
            .address(generate_random_iota_address())
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
        input_transaction_id: TransactionId,
        input_address: Option<AddressWrapper>,
        account_addresses: Vec<Address>,
    }

    impl Default for GenerateMessageBuilder {
        fn default() -> Self {
            Self {
                value: rand::thread_rng().gen_range(1..50000),
                address: generate_random_address(),
                confirmed: Some(false),
                broadcasted: false,
                input_transaction_id: TransactionId::new([0; 32]),
                input_address: None,
                account_addresses: Vec::new(),
            }
        }
    }

    builder_setters!(
        GenerateMessageBuilder,
        value => u64,
        address => Address,
        confirmed => Option<bool>,
        broadcasted => bool,
        input_transaction_id => TransactionId,
        input_address => Option<AddressWrapper>,
        account_addresses => Vec<Address>
    );

    impl GenerateMessageBuilder {
        pub async fn build(self) -> Message {
            let bech32_hrp = self.address.address().bech32_hrp().to_string();
            let mut id = [0; 32];
            crypto::utils::rand::fill(&mut id).unwrap();
            let id = MessageId::new(id);
            let tx_metadata = TransactionBuilderMetadata {
                id: &id,
                bech32_hrp,
                account_id: "",
                accounts: AccountStore::new(Default::default()),
                account_addresses: &self.account_addresses,
                client_options: &ClientOptionsBuilder::new().build().unwrap(),
            };

            let mut payload = MessagePayload::new(
                Payload::Transaction(Box::new(
                    TransactionPayloadBuilder::new()
                        .with_essence(Essence::Regular(
                            iota_client::bee_message::prelude::RegularEssence::builder()
                                .add_output(
                                    SignatureLockedSingleOutput::new(*self.address.address().as_ref(), self.value)
                                        .unwrap()
                                        .into(),
                                )
                                .add_input(UtxoInput::new(self.input_transaction_id, 0).unwrap().into())
                                .finish()
                                .unwrap(),
                        ))
                        .with_unlock_blocks(
                            UnlockBlocks::new(vec![UnlockBlock::Signature(SignatureUnlock::Ed25519(
                                Ed25519Signature::new([0; 32], [0; 64]),
                            ))])
                            .unwrap(),
                        )
                        .finish()
                        .unwrap(),
                )),
                &tx_metadata,
            )
            .await
            .unwrap();
            if let MessagePayload::Transaction(ref mut tx) = payload {
                let TransactionEssence::Regular(ref mut essence) = tx.essence_mut();
                if let Some(address) = self.input_address {
                    let input = essence.inputs_mut().iter_mut().next().unwrap();
                    if let crate::message::TransactionInput::Utxo(ref mut utxo) = input {
                        utxo.metadata.replace(crate::address::AddressOutput {
                            transaction_id: self.input_transaction_id,
                            message_id: iota_client::bee_message::MessageId::from([0; 32]),
                            index: 0,
                            amount: 10000000,
                            is_spent: false,
                            address,
                            kind: crate::address::OutputKind::SignatureLockedSingle,
                        });
                        essence.incoming = essence.is_incoming(&self.account_addresses);
                    }
                }
            }

            Message {
                id,
                version: 1,
                parents: vec![MessageId::new([0; 32])],
                payload_length: 0,
                payload: Some(payload),
                timestamp: chrono::Utc::now(),
                nonce: 0,
                confirmed: self.confirmed,
                broadcasted: self.broadcasted,
                reattachment_message_id: None,
            }
        }
    }
}
