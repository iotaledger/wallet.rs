//! The IOTA Wallet Library

#![warn(missing_docs, rust_2018_idioms)]
#![allow(unused_variables, dead_code)]

/// The account module.
pub mod account;
/// The account manager module.
pub mod account_manager;
/// The address module.
pub mod address;
/// The client module.
pub mod client;
/// The event module.
pub mod event;
/// The message module.
pub mod message;
/// The monitor module.
pub mod monitor;
/// The storage module.
pub mod storage;

pub use anyhow::Result;
pub use chrono::prelude::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use stronghold::Stronghold;

static STRONGHOLD_INSTANCE: OnceCell<Arc<Mutex<HashMap<PathBuf, Stronghold>>>> = OnceCell::new();

pub(crate) fn init_stronghold(stronghold_path: PathBuf, stronghold: Stronghold) {
    let mut stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.insert(stronghold_path, stronghold);
}

pub(crate) fn remove_stronghold(stronghold_path: PathBuf) {
    let mut stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    stronghold_map.remove(&stronghold_path);
}

pub(crate) fn with_stronghold<T, F: FnOnce(&Stronghold) -> T>(cb: F) -> T {
    with_stronghold_from_path(&crate::storage::get_stronghold_snapshot_path(), cb)
}

pub(crate) fn with_stronghold_from_path<T, F: FnOnce(&Stronghold) -> T>(
    path: &PathBuf,
    cb: F,
) -> T {
    let stronghold_map = STRONGHOLD_INSTANCE
        .get_or_init(Default::default)
        .lock()
        .unwrap();
    if let Some(stronghold) = stronghold_map.get(path) {
        cb(stronghold)
    } else {
        panic!("should initialize stronghold instance before using it")
    }
}

#[cfg(test)]
mod test_utils {
    use super::account::Account;
    use super::account_manager::AccountManager;
    use super::address::{Address, IotaAddress};
    use super::client::ClientOptionsBuilder;
    use super::message::Message;

    use chrono::prelude::Utc;
    use iota::transaction::prelude::{
        Ed25519Address, MessageId, Payload, Seed, SignatureLockedSingleOutput, TransactionBuilder,
        UTXOInput,
    };
    use once_cell::sync::OnceCell;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use slip10::BIP32Path;

    use std::convert::TryInto;
    use std::num::NonZeroU64;
    use std::path::PathBuf;

    static MANAGER_INSTANCE: OnceCell<AccountManager> = OnceCell::new();
    pub fn get_account_manager() -> &'static AccountManager {
        let storage_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
        let storage_path = PathBuf::from(format!("./example-database/{}", storage_path));
        crate::storage::set_storage_path(&storage_path).unwrap();

        MANAGER_INSTANCE.get_or_init(|| {
            let manager = AccountManager::new();
            manager.set_stronghold_password("password").unwrap();
            manager
        })
    }

    pub fn create_account(manager: &AccountManager, addresses: Vec<Address>) -> Account {
        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        manager
            .create_account(client_options)
            .alias("alias")
            .initialise()
            .expect("failed to add account")
    }

    pub fn generate_random_iota_address() -> IotaAddress {
        IotaAddress::Ed25519(Ed25519Address::new(rand::random::<[u8; 32]>()))
    }

    pub fn generate_message(
        value: i64,
        address: Address,
        confirmed: bool,
        broadcasted: bool,
    ) -> Message {
        Message {
            version: 1,
            trunk: MessageId::new([0; 32]),
            branch: MessageId::new([0; 32]),
            payload_length: 0,
            payload: Payload::Transaction(Box::new(
                TransactionBuilder::new(&Seed::from_ed25519_bytes("".as_bytes()).unwrap())
                    .set_outputs(vec![SignatureLockedSingleOutput::new(
                        address.address().clone(),
                        NonZeroU64::new(value.try_into().unwrap()).unwrap(),
                    )
                    .into()])
                    .set_inputs(vec![(
                        UTXOInput::new(MessageId::new([0; 32]), 0).unwrap().into(),
                        BIP32Path::from_str("").unwrap(),
                    )])
                    .build()
                    .unwrap(),
            )),
            timestamp: Utc::now(),
            nonce: 0,
            confirmed,
            broadcasted,
        }
    }
}
