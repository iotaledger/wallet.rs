// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Stronghold interface abstractions over an account

use crate::account::AccountIdentifier;
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use iota::{Address, Ed25519Address, Ed25519Signature};
use iota_stronghold::{Location, ProcResult, Procedure, RecordHint, ResultMessage, Stronghold, StrongholdFlags};
use once_cell::sync::{Lazy, OnceCell};
use riker::actors::*;
use tokio::{
    sync::Mutex,
    task,
    time::{delay_for, Duration},
};

use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

type SnapshotToPasswordMap = HashMap<PathBuf, [u8; 32]>;
static PASSWORD_STORE: OnceCell<Arc<Mutex<SnapshotToPasswordMap>>> = OnceCell::new();
static STRONGHOLD_ACCESS_STORE: OnceCell<Arc<Mutex<HashMap<PathBuf, bool>>>> = OnceCell::new();
static CURRENT_SNAPSHOT_PATH: OnceCell<Arc<Mutex<Option<PathBuf>>>> = OnceCell::new();
static PASSWORD_CLEAR_INTERVAL: OnceCell<Arc<Mutex<Duration>>> = OnceCell::new();
static PRIVATE_DATA_CLIENT_PATH: &[u8] = b"iota_seed";

const TIMEOUT: Duration = Duration::from_millis(5000);
#[cfg(test)]
const DEFAULT_PASSWORD_CLEAR_INTERVAL: Duration = Duration::from_secs(0);
#[cfg(not(test))]
const DEFAULT_PASSWORD_CLEAR_INTERVAL: Duration = Duration::from_secs(8 * 60);
const ACCOUNT_ID_SEPARATOR: char = '|';
const ACCOUNT_METADATA_VAULT_PATH: &str = "iota-wallet-account-metadata";
const ACCOUNT_IDS_RECORD_PATH: &str = "iota-wallet-account-ids";
const ACCOUNT_VAULT_PATH: &str = "iota-wallet-account-vault";
const ACCOUNT_RECORD_PATH: &str = "iota-wallet-account-record";
const SECRET_VAULT_PATH: &str = "iota-wallet-secret";
const SEED_RECORD_PATH: &str = "iota-wallet-seed";
const DERIVE_OUTPUT_RECORD_PATH: &str = "iota-wallet-derived";

/// The default stronghold storage file name.
pub const SNAPSHOT_FILENAME: &str = "wallet.stronghold";

fn account_id_to_client_path(id: &AccountIdentifier) -> Vec<u8> {
    match id {
        AccountIdentifier::Id(id) => format!("iota-wallet-account-{}", id).as_bytes().to_vec(),
        _ => unreachable!(),
    }
}

fn stronghold_response_to_result<T>(status: ResultMessage<T>) -> Result<T> {
    match status {
        ResultMessage::Ok(v) => Ok(v),
        ResultMessage::Error(e) => Err(Error::FailedToPerformAction(e)),
    }
}

async fn load_actor(
    runtime: &mut ActorRuntime,
    snapshot_path: &PathBuf,
    client_path: Vec<u8>,
    flags: Vec<StrongholdFlags>,
) -> Result<()> {
    on_stronghold_access(&snapshot_path).await?;

    if runtime.spawned_client_paths.contains(&client_path) {
        stronghold_response_to_result(runtime.stronghold.switch_actor_target(client_path.clone()))?;
    } else {
        stronghold_response_to_result(runtime.stronghold.spawn_stronghold_actor(client_path.clone(), flags))?;
        runtime.spawned_client_paths.insert(client_path.clone());
    };

    if !runtime.loaded_client_paths.contains(&client_path) {
        let snapshot_file_path = snapshot_path.join(SNAPSHOT_FILENAME);
        if snapshot_file_path.exists() {
            stronghold_response_to_result(
                runtime
                    .stronghold
                    .read_snapshot(
                        client_path.clone(),
                        None,
                        get_password(snapshot_path).await?.to_vec(),
                        None,
                        Some(snapshot_file_path),
                    )
                    .await,
            )?;
        }
        runtime.loaded_client_paths.insert(client_path);
    }

    Ok(())
}

async fn load_private_data_actor(runtime: &mut ActorRuntime, snapshot_path: &PathBuf) -> Result<()> {
    load_actor(
        runtime,
        snapshot_path,
        PRIVATE_DATA_CLIENT_PATH.to_vec(),
        vec![StrongholdFlags::IsReadable(false)],
    )
    .await
}

async fn load_account_actor(
    runtime: &mut ActorRuntime,
    snapshot_path: &PathBuf,
    account_id: &AccountIdentifier,
) -> Result<()> {
    load_actor(
        runtime,
        snapshot_path,
        account_id_to_client_path(account_id),
        vec![StrongholdFlags::IsReadable(true)],
    )
    .await
}

async fn on_stronghold_access<S: AsRef<Path>>(snapshot_path: S) -> Result<()> {
    let mut store = STRONGHOLD_ACCESS_STORE.get_or_init(Default::default).lock().await;
    store.insert(snapshot_path.as_ref().to_path_buf(), true);

    let passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    if !passwords.contains_key(&snapshot_path.as_ref().to_path_buf()) {
        Err(Error::PasswordNotSet)
    } else {
        Ok(())
    }
}

pub async fn set_password_clear_interval(interval: Duration) {
    let mut clear_interval = PASSWORD_CLEAR_INTERVAL
        .get_or_init(|| Arc::new(Mutex::new(DEFAULT_PASSWORD_CLEAR_INTERVAL)))
        .lock()
        .await;
    *clear_interval = interval;
}

fn default_password_store() -> Arc<Mutex<HashMap<PathBuf, [u8; 32]>>> {
    thread::spawn(|| {
        crate::enter(|| {
            task::spawn(async {
                let interval = *PASSWORD_CLEAR_INTERVAL
                    .get_or_init(|| Arc::new(Mutex::new(DEFAULT_PASSWORD_CLEAR_INTERVAL)))
                    .lock()
                    .await;
                loop {
                    delay_for(interval).await;

                    if interval.as_nanos() == 0 {
                        continue;
                    }

                    let mut passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
                    let mut access_store = STRONGHOLD_ACCESS_STORE.get_or_init(Default::default).lock().await;
                    let mut remove_keys = Vec::new();
                    for (snapshot_path, _) in passwords.iter() {
                        // if the stronghold access flag is false,
                        if !*access_store.get(snapshot_path).unwrap_or(&true) {
                            remove_keys.push(snapshot_path.clone());
                        }
                        access_store.insert(snapshot_path.clone(), false);
                    }

                    let current_snapshot_path = &*CURRENT_SNAPSHOT_PATH.get_or_init(Default::default).lock().await;
                    for remove_key in remove_keys {
                        passwords.remove(&remove_key);
                        if let Some(curr_snapshot_path) = current_snapshot_path {
                            if &remove_key == curr_snapshot_path {
                                let mut runtime = actor_runtime().lock().await;
                                let _ = clear_stronghold_cache(&mut runtime);
                            }
                        }
                    }
                }
            })
        })
    });
    Default::default()
}

pub async fn set_password<S: AsRef<Path>>(snapshot_path: S, password: &[u8; 32]) {
    let mut passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    passwords.insert(snapshot_path.as_ref().to_path_buf(), *password);
}

async fn get_password<P: AsRef<Path>>(snapshot_path: P) -> Result<[u8; 32]> {
    let passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    passwords
        .get(&snapshot_path.as_ref().to_path_buf())
        .cloned()
        .ok_or(Error::PasswordNotSet)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("account id isn't a valid record hint")]
    InvalidAccountIdentifier,
    #[error("must provide account id instead of string")]
    AccountIdMustBeString,
    #[error("`{0}`")]
    StrongholdError(#[from] iota_stronghold::Error),
    #[error("account not found")]
    AccountNotFound,
    #[error("snapshot doesn't have accounts")]
    EmptySnapshot,
    #[error("failed to perform action: `{0}`")]
    FailedToPerformAction(String),
    #[error("snapshot password not set")]
    PasswordNotSet,
    #[error("failed to create snapshot directory")]
    FailedToCreateSnapshotDir,
    #[error("invalid address or account index {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ActorRuntime {
    pub stronghold: Stronghold,
    spawned_client_paths: HashSet<Vec<u8>>,
    loaded_client_paths: HashSet<Vec<u8>>,
}

fn system_runtime() -> &'static Arc<Mutex<ActorSystem>> {
    static SYSTEM: Lazy<Arc<Mutex<ActorSystem>>> = Lazy::new(|| {
        let system = ActorSystem::new().unwrap();
        Arc::new(Mutex::new(system))
    });
    &SYSTEM
}

pub fn actor_runtime() -> &'static Arc<Mutex<ActorRuntime>> {
    static SYSTEM: Lazy<Arc<Mutex<ActorRuntime>>> = Lazy::new(|| {
        let system = ActorSystem::new().unwrap();
        let stronghold = Stronghold::init_stronghold_system(
            system,
            PRIVATE_DATA_CLIENT_PATH.to_vec(),
            vec![StrongholdFlags::IsReadable(false)],
        );

        let mut spawned_client_paths = HashSet::new();
        spawned_client_paths.insert(PRIVATE_DATA_CLIENT_PATH.to_vec());

        let runtime = ActorRuntime {
            stronghold,
            spawned_client_paths,
            loaded_client_paths: HashSet::new(),
        };
        Arc::new(Mutex::new(runtime))
    });
    &SYSTEM
}

// check if the snapshot path is different than the current loaded one
// if it is, write the current snapshot and load the new one
async fn check_snapshot(mut runtime: &mut ActorRuntime, snapshot_path: &PathBuf) -> Result<()> {
    let curr_snapshot_path = CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .lock()
        .await
        .as_ref()
        .cloned();

    if let Some(curr_snapshot_path) = &curr_snapshot_path {
        // if the current loaded snapshot is different than the snapshot we're tring to use,
        // save the current snapshot and clear the cache
        if curr_snapshot_path != snapshot_path {
            switch_snapshot(&mut runtime, snapshot_path).await?;
        }
    }

    Ok(())
}

// saves the snapshot to the file system.
async fn save_snapshot(runtime: &mut ActorRuntime, snapshot_path: &PathBuf) -> Result<()> {
    stronghold_response_to_result(
        runtime
            .stronghold
            .write_all_to_snapshot(
                get_password(snapshot_path).await?.to_vec(),
                None,
                Some(snapshot_path.join(SNAPSHOT_FILENAME)),
            )
            .await,
    )
}

async fn clear_stronghold_cache(mut runtime: &mut ActorRuntime) -> Result<()> {
    if let Some(curr_snapshot_path) = CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .lock()
        .await
        .as_ref()
    {
        if !runtime.spawned_client_paths.is_empty() {
            save_snapshot(&mut runtime, &curr_snapshot_path).await?;
        }
        for path in &runtime.spawned_client_paths {
            stronghold_response_to_result(runtime.stronghold.kill_stronghold(path.clone(), false).await)?;
            stronghold_response_to_result(runtime.stronghold.kill_stronghold(path.clone(), true).await)?;
        }
        // delay to wait for the actors to be killed
        thread::sleep(std::time::Duration::from_millis(300));
        runtime.spawned_client_paths = HashSet::new();
        runtime.loaded_client_paths = HashSet::new();
    }

    Ok(())
}

async fn switch_snapshot(mut runtime: &mut ActorRuntime, snapshot_path: &PathBuf) -> Result<()> {
    clear_stronghold_cache(&mut runtime).await?;

    let mut current_snapshot_path = CURRENT_SNAPSHOT_PATH.get_or_init(Default::default).lock().await;
    current_snapshot_path.replace(snapshot_path.clone());

    // load all actors to prevent lost data on save
    load_private_data_actor(&mut runtime, snapshot_path).await?;
    let account_ids_location = Location::generic(ACCOUNT_METADATA_VAULT_PATH, ACCOUNT_IDS_RECORD_PATH);
    let (data_opt, status) = runtime.stronghold.read_data(account_ids_location.clone()).await;

    if let Some(data) = data_opt {
        let account_ids = String::from_utf8_lossy(&data).to_string();
        for account_id in account_ids.split(ACCOUNT_ID_SEPARATOR).filter(|id| !id.is_empty()) {
            let id = AccountIdentifier::Id(account_id.to_string());
            load_account_actor(&mut runtime, snapshot_path, &id).await?;
        }
    }

    Ok(())
}

pub async fn load_snapshot(snapshot_path: &PathBuf, password: &[u8; 32]) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    std::fs::create_dir_all(&snapshot_path).map_err(|_| Error::FailedToCreateSnapshotDir)?;
    set_password(&snapshot_path, password).await;
    switch_snapshot(&mut runtime, &snapshot_path).await
}

pub async fn store_mnemonic(snapshot_path: &PathBuf, mnemonic: String) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path).await?;
    load_private_data_actor(&mut runtime, snapshot_path).await?;

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::BIP39Recover {
            mnemonic,
            passphrase: None,
            output: Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH),
            hint: RecordHint::new("wallet.rs-seed").unwrap(),
        })
        .await;

    if let ProcResult::BIP39Recover(status) = res {
        stronghold_response_to_result(status)?;
        save_snapshot(&mut runtime, snapshot_path).await
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

pub async fn generate_address(
    snapshot_path: &PathBuf,
    account_index: usize,
    address_index: usize,
    internal: bool,
) -> Result<Address> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, &snapshot_path).await?;
    load_private_data_actor(&mut runtime, snapshot_path).await?;

    let chain = format!("m/44H/4218H/{}H/{}H/{}H", account_index, internal as u32, address_index,);

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::Ed25519PublicKey {
            key: Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH),
            path: chain,
        })
        .await;
    if let ProcResult::Ed25519PublicKey(response) = res {
        let public_key = stronghold_response_to_result(response)?;
        // Hash the public key to get the address
        let mut hasher = VarBlake2b::new(32).unwrap();
        hasher.update(public_key);
        let mut result = vec![];
        hasher.finalize_variable(|res| {
            result = res.to_vec();
        });
        Ok(Address::Ed25519(Ed25519Address::new(result.try_into().unwrap())))
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

pub async fn sign_essence(
    snapshot_path: &PathBuf,
    transaction_essence: Vec<u8>,
    account_index: usize,
    address_index: usize,
    internal: bool,
) -> Result<Ed25519Signature> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, &snapshot_path).await?;
    load_private_data_actor(&mut runtime, snapshot_path).await?;

    let chain = format!("m/44H/4218H/{}H/{}H/{}H", account_index, internal as u32, address_index,);

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::SignUnlockBlock {
            path: chain,
            seed: Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH),
            essence: transaction_essence,
        })
        .await;
    if let ProcResult::SignUnlockBlock(response) = res {
        let (signature, public_key) = stronghold_response_to_result(response)?;
        Ok(Ed25519Signature::new(public_key, Box::new(signature)))
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

pub async fn get_accounts(snapshot_path: &PathBuf) -> Result<Vec<String>> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, &snapshot_path).await?;
    load_private_data_actor(&mut runtime, snapshot_path).await?;
    let account_ids_location = Location::generic(ACCOUNT_METADATA_VAULT_PATH, ACCOUNT_IDS_RECORD_PATH);
    let (data_opt, status) = runtime.stronghold.read_data(account_ids_location.clone()).await;

    let mut accounts = Vec::new();
    if let Some(data) = data_opt {
        let account_ids = String::from_utf8_lossy(&data).to_string();
        for account_id in account_ids.split(ACCOUNT_ID_SEPARATOR).filter(|id| !id.is_empty()) {
            let id = AccountIdentifier::Id(account_id.to_string());
            let account = get_account_internal(&mut runtime, snapshot_path, &id).await?;
            accounts.push(account);
        }
    }

    Ok(accounts)
}

async fn get_account_internal(
    mut runtime: &mut ActorRuntime,
    snapshot_path: &PathBuf,
    account_id: &AccountIdentifier,
) -> Result<String> {
    load_account_actor(&mut runtime, snapshot_path, account_id).await?;
    let (data, _) = runtime
        .stronghold
        .read_data(Location::generic(ACCOUNT_VAULT_PATH, ACCOUNT_RECORD_PATH))
        .await;
    data.filter(|data| !data.is_empty())
        .map(|data| String::from_utf8_lossy(&data).to_string())
        .ok_or(Error::AccountNotFound)
}

pub async fn get_account(snapshot_path: &PathBuf, account_id: &AccountIdentifier) -> Result<String> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, &snapshot_path).await?;
    get_account_internal(&mut runtime, snapshot_path, account_id).await
}

pub async fn store_account(snapshot_path: &PathBuf, account_id: &AccountIdentifier, account: String) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, &snapshot_path).await?;

    // first we push the account id to the reference array
    // the reference array holds all account ids so we can scan them on the `get_accounts` implementation
    load_private_data_actor(&mut runtime, snapshot_path).await?;
    let account_ids_location = Location::generic(ACCOUNT_METADATA_VAULT_PATH, ACCOUNT_IDS_RECORD_PATH);
    let (data_opt, status) = runtime.stronghold.read_data(account_ids_location.clone()).await;
    let mut account_ids = data_opt
        .map(|d| String::from_utf8_lossy(&d).to_string())
        .unwrap_or_else(String::new);

    let id = match account_id {
        AccountIdentifier::Id(id) => id,
        AccountIdentifier::Index(_) => unreachable!(),
    };
    if !account_ids.contains(id.as_str()) {
        account_ids.push_str(id);
        stronghold_response_to_result(
            runtime
                .stronghold
                .write_data(
                    account_ids_location,
                    account_ids.as_bytes().to_vec(),
                    RecordHint::new("wallet.rs-account-ids").unwrap(),
                    vec![],
                )
                .await,
        )?;
    }

    load_account_actor(&mut runtime, snapshot_path, account_id).await?;
    stronghold_response_to_result(
        runtime
            .stronghold
            .write_data(
                Location::generic(ACCOUNT_VAULT_PATH, ACCOUNT_RECORD_PATH),
                account.as_bytes().to_vec(),
                RecordHint::new("wallet.rs-account").unwrap(),
                vec![],
            )
            .await,
    )?;

    save_snapshot(&mut runtime, snapshot_path).await?;

    Ok(())
}

pub async fn remove_account(snapshot_path: &PathBuf, account_id: &AccountIdentifier) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, &snapshot_path).await?;

    // first we delete the account id from the reference array
    load_private_data_actor(&mut runtime, snapshot_path).await?;
    let account_ids_location = Location::generic(ACCOUNT_METADATA_VAULT_PATH, ACCOUNT_IDS_RECORD_PATH);
    let (data_opt, status) = runtime.stronghold.read_data(account_ids_location.clone()).await;
    let account_ids = data_opt
        .filter(|data| !data.is_empty())
        .map(|data| String::from_utf8_lossy(&data).to_string())
        .ok_or(Error::AccountNotFound)?;
    stronghold_response_to_result(
        runtime
            .stronghold
            .write_data(
                account_ids_location,
                account_ids.as_bytes().to_vec(),
                RecordHint::new("wallet.rs-account-ids").unwrap(),
                vec![],
            )
            .await,
    )?;

    load_account_actor(&mut runtime, snapshot_path, account_id).await?;
    stronghold_response_to_result(
        runtime
            .stronghold
            .delete_data(Location::generic(ACCOUNT_VAULT_PATH, ACCOUNT_RECORD_PATH), true)
            .await,
    )?;

    save_snapshot(&mut runtime, snapshot_path).await
}

#[cfg(test)]
mod tests {
    use crate::account::AccountIdentifier;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use rusty_fork::rusty_fork_test;
    use std::path::PathBuf;
    use tokio::time::Duration;

    rusty_fork_test! {
        #[test]
        fn password_expires() {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let interval = 500;
                super::set_password_clear_interval(Duration::from_millis(interval)).await;
                let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
                let snapshot_path = PathBuf::from(format!("./test-storage/{}", snapshot_path));
                super::load_snapshot(&snapshot_path, &[0; 32]).await.unwrap();

                std::thread::sleep(Duration::from_millis(interval * 3));
                let res = super::get_account(&snapshot_path, &AccountIdentifier::Id("passwordexpires".to_string())).await;
                assert_eq!(res.is_err(), true);
                let error = res.unwrap_err();
                if let super::Error::PasswordNotSet = error {
                } else {
                    panic!("unexpected error: {:?}", error);
                }
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn action_keeps_password() {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let interval = Duration::from_millis(500);
                super::set_password_clear_interval(interval).await;
                let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
                let snapshot_path = PathBuf::from(format!("./test-storage/{}", snapshot_path));
                super::load_snapshot(&snapshot_path, &[0; 32]).await.unwrap();

                for i in 1..5 {
                    super::store_account(
                        &snapshot_path,
                        &AccountIdentifier::Id(format!("actionkeepspassword{}", i)),
                        "data".to_string(),
                    )
                    .await
                    .unwrap();
                    std::thread::sleep(interval / 2);
                }

                let id = AccountIdentifier::Id("actionkeepspassword1".to_string());
                let res = super::get_account(&snapshot_path, &id).await;
                assert_eq!(res.is_ok(), true);

                std::thread::sleep(interval * 2);

                let res = super::get_account(&snapshot_path, &id).await;
                assert_eq!(res.is_err(), true);
                if let super::Error::PasswordNotSet = res.unwrap_err() {
                } else {
                    panic!("unexpected error");
                }
            });
        }
    }

    #[tokio::test]
    async fn write_and_get_all() -> super::Result<()> {
        let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, &[0; 32]).await?;

        let id = AccountIdentifier::Id("id".to_string());
        let data = "account data";
        super::store_account(&snapshot_path, &id, data.to_string()).await?;
        let mut r = super::actor_runtime().lock().await;
        super::clear_stronghold_cache(&mut r).await?;
        drop(r);
        let stored_data = super::get_accounts(&snapshot_path).await?;
        assert_eq!(stored_data.len(), 1);
        assert_eq!(stored_data[0], data);

        Ok(())
    }

    #[tokio::test]
    async fn write_and_read() -> super::Result<()> {
        let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, &[0; 32]).await?;

        let id = AccountIdentifier::Id("writeandreadtest".to_string());
        let data = "account data";
        super::store_account(&snapshot_path, &id, data.to_string()).await?;
        let stored_data = super::get_account(&snapshot_path, &id).await?;
        assert_eq!(stored_data, data);

        Ok(())
    }

    #[tokio::test]
    async fn write_and_delete() -> super::Result<()> {
        let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, &[0; 32]).await?;

        let id = AccountIdentifier::Id("writeanddeleteid".to_string());
        let data = "account data";
        super::store_account(&snapshot_path, &id, data.to_string()).await?;
        super::remove_account(&snapshot_path, &id).await?;

        Ok(())
    }

    #[tokio::test]
    async fn write_and_read_multiple_snapshots() -> super::Result<()> {
        let mut snapshot_saves = vec![];

        for i in 1..3 {
            let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
            let snapshot_path = PathBuf::from(format!("./test-storage/{}", snapshot_path));
            super::load_snapshot(&snapshot_path, &[0; 32]).await?;

            let id = AccountIdentifier::Id(format!("multiplesnapshots{}", i));
            let data: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
            super::store_account(&snapshot_path, &id, data.clone()).await?;
            snapshot_saves.push((snapshot_path, id, data));
        }

        for (snapshot_path, account_id, data) in snapshot_saves {
            let stored_data = super::get_account(&snapshot_path, &account_id).await?;
            assert_eq!(stored_data, data);
        }

        Ok(())
    }
}
