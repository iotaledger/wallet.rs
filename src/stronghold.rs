// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Stronghold interface abstractions over an account

use crypto::hashes::{blake2b::Blake2b256, Digest};

use crypto::keys::slip10::Chain;
use getset::Getters;
use iota_client::bee_message::prelude::{Address, Ed25519Address, Ed25519Signature};
use iota_stronghold::{
    Location, ProcResult, Procedure, RecordHint, ResultMessage, SLIP10DeriveInput, Stronghold, StrongholdFlags,
};
use once_cell::sync::{Lazy, OnceCell};
use riker::actors::*;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    num::TryFromIntError,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Instant,
};
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};
use zeroize::Zeroize;

#[derive(PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
struct Password(Vec<u8>);

type SnapshotToPasswordMap = HashMap<PathBuf, Arc<Password>>;
static PASSWORD_STORE: OnceCell<Arc<Mutex<SnapshotToPasswordMap>>> = OnceCell::new();
static STRONGHOLD_ACCESS_STORE: OnceCell<Arc<Mutex<HashMap<PathBuf, Instant>>>> = OnceCell::new();
static CURRENT_SNAPSHOT_PATH: OnceCell<Arc<Mutex<Option<PathBuf>>>> = OnceCell::new();
static PASSWORD_CLEAR_INTERVAL: OnceCell<Arc<Mutex<Duration>>> = OnceCell::new();
static PRIVATE_DATA_CLIENT_PATH: &[u8] = b"iota_seed";

const DEFAULT_PASSWORD_CLEAR_INTERVAL: Duration = Duration::from_secs(0);
const SECRET_VAULT_PATH: &str = "iota-wallet-secret";
const SEED_RECORD_PATH: &str = "iota-wallet-seed";
const DERIVE_OUTPUT_RECORD_PATH: &str = "iota-wallet-derived";

fn records_client_path() -> Vec<u8> {
    b"iota-wallet-records".to_vec()
}

fn stronghold_response_to_result<T>(status: ResultMessage<T>) -> Result<T> {
    match status {
        ResultMessage::Ok(v) => Ok(v),
        ResultMessage::Error(e) => Err(Error::FailedToPerformAction(e)),
    }
}

async fn load_actor(
    runtime: &mut ActorRuntime,
    snapshot_path: &Path,
    client_path: Vec<u8>,
    flags: Vec<StrongholdFlags>,
    password: Option<Arc<Password>>,
) -> Result<()> {
    if password.is_none() {
        on_stronghold_access(&snapshot_path).await?;
    }

    if runtime.spawned_client_paths.contains(&client_path) {
        stronghold_response_to_result(runtime.stronghold.switch_actor_target(client_path.clone()).await)?;
    } else {
        stronghold_response_to_result(
            runtime
                .stronghold
                .spawn_stronghold_actor(client_path.clone(), flags)
                .await,
        )?;
        runtime.spawned_client_paths.insert(client_path.clone());
    };

    if !runtime.loaded_client_paths.contains(&client_path) {
        if snapshot_path.exists() {
            stronghold_response_to_result(
                runtime
                    .stronghold
                    .read_snapshot(
                        client_path.clone(),
                        None,
                        &get_password_if_needed(snapshot_path, password).await?.0,
                        None,
                        Some(snapshot_path.to_path_buf()),
                    )
                    .await,
            )?;
        }
        runtime.loaded_client_paths.insert(client_path);
    }

    Ok(())
}

async fn load_private_data_actor(
    runtime: &mut ActorRuntime,
    snapshot_path: &Path,
    password: Option<Arc<Password>>,
) -> Result<()> {
    load_actor(
        runtime,
        snapshot_path,
        PRIVATE_DATA_CLIENT_PATH.to_vec(),
        vec![StrongholdFlags::IsReadable(false)],
        password,
    )
    .await
}

async fn load_records_actor(
    runtime: &mut ActorRuntime,
    snapshot_path: &Path,
    password: Option<Arc<Password>>,
) -> Result<()> {
    load_actor(
        runtime,
        snapshot_path,
        records_client_path(),
        vec![StrongholdFlags::IsReadable(true)],
        password,
    )
    .await
}

async fn on_stronghold_access<S: AsRef<Path>>(snapshot_path: S) -> Result<()> {
    let passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    if !passwords.contains_key(&snapshot_path.as_ref().to_path_buf()) {
        Err(Error::PasswordNotSet)
    } else {
        let mut store = STRONGHOLD_ACCESS_STORE.get_or_init(Default::default).lock().await;
        store.insert(snapshot_path.as_ref().to_path_buf(), Instant::now());
        Ok(())
    }
}

/// Set the password clear interval.
/// If the stronghold isn't used after `interval`, the password is cleared and must be set again.
pub async fn set_password_clear_interval(interval: Duration) {
    let mut clear_interval = PASSWORD_CLEAR_INTERVAL
        .get_or_init(|| Arc::new(Mutex::new(DEFAULT_PASSWORD_CLEAR_INTERVAL)))
        .lock()
        .await;
    *clear_interval = interval;
}

/// Snapshot status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "data")]
pub enum SnapshotStatus {
    /// Snapshot is locked. This means that the password must be set again.
    Locked,
    /// Snapshot is unlocked. The duration is the amount of time left before it locks again.
    Unlocked(Duration),
}

#[derive(Clone, Getters, Debug, Serialize)]
#[getset(get = "pub")]
/// Stronghold status.
pub struct Status {
    /// The snapshot path.
    #[serde(rename = "snapshotPath")]
    pub snapshot_path: PathBuf,
    /// The snapshot status.
    pub snapshot: SnapshotStatus,
}

/// Gets the stronghold status for the given snapshot.
pub async fn get_status(snapshot_path: &Path) -> Status {
    let password_clear_interval = *PASSWORD_CLEAR_INTERVAL
        .get_or_init(|| Arc::new(Mutex::new(DEFAULT_PASSWORD_CLEAR_INTERVAL)))
        .lock()
        .await;
    if let Some(access_instant) = STRONGHOLD_ACCESS_STORE
        .get_or_init(Default::default)
        .lock()
        .await
        .get(snapshot_path)
    {
        let locked = password_clear_interval.as_millis() > 0 && access_instant.elapsed() >= password_clear_interval;
        Status {
            snapshot_path: snapshot_path.to_path_buf(),
            snapshot: if locked {
                SnapshotStatus::Locked
            } else {
                SnapshotStatus::Unlocked(if password_clear_interval.as_millis() == 0 {
                    password_clear_interval
                } else {
                    password_clear_interval - access_instant.elapsed()
                })
            },
        }
    } else {
        Status {
            snapshot_path: snapshot_path.to_path_buf(),
            snapshot: SnapshotStatus::Locked,
        }
    }
}

fn default_password_store() -> Arc<Mutex<HashMap<PathBuf, Arc<Password>>>> {
    thread::spawn(|| {
        crate::spawn(async {
            loop {
                let interval = *PASSWORD_CLEAR_INTERVAL
                    .get_or_init(|| Arc::new(Mutex::new(DEFAULT_PASSWORD_CLEAR_INTERVAL)))
                    .lock()
                    .await;
                sleep(interval).await;

                if interval.as_nanos() == 0 {
                    continue;
                }

                let mut passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
                let access_store = STRONGHOLD_ACCESS_STORE.get_or_init(Default::default).lock().await;
                let mut snapshots_paths_to_clear = Vec::new();
                for (snapshot_path, _) in passwords.iter() {
                    // if the stronghold was accessed `interval` ago, we clear the password
                    if let Some(access_instant) = access_store.get(snapshot_path) {
                        if access_instant.elapsed() > interval {
                            snapshots_paths_to_clear.push(snapshot_path.clone());
                        }
                    }
                }

                let current_snapshot_path = &*CURRENT_SNAPSHOT_PATH.get_or_init(Default::default).lock().await;
                for snapshot_path in snapshots_paths_to_clear {
                    passwords.remove(&snapshot_path);
                    if let Some(curr_snapshot_path) = current_snapshot_path {
                        if &snapshot_path == curr_snapshot_path {
                            let mut runtime = actor_runtime().lock().await;
                            let _ = clear_stronghold_cache(&mut runtime, true);
                        }
                    }
                    crate::event::emit_stronghold_status_change(&Status {
                        snapshot_path: snapshot_path.clone(),
                        snapshot: SnapshotStatus::Locked,
                    })
                    .await;
                }
            }
        })
    });
    Default::default()
}

pub async fn set_password<S: AsRef<Path>>(snapshot_path: S, password: Vec<u8>) {
    let mut passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    let mut access_store = STRONGHOLD_ACCESS_STORE.get_or_init(Default::default).lock().await;

    let snapshot_path = snapshot_path.as_ref().to_path_buf();
    access_store.insert(snapshot_path.clone(), Instant::now());
    passwords.insert(snapshot_path, Arc::new(Password(password)));
}

async fn get_password_if_needed(snapshot_path: &Path, password: Option<Arc<Password>>) -> Result<Arc<Password>> {
    match password {
        Some(password) => Ok(password),
        None => get_password(snapshot_path).await,
    }
}

async fn get_password(snapshot_path: &Path) -> Result<Arc<Password>> {
    PASSWORD_STORE
        .get_or_init(default_password_store)
        .lock()
        .await
        .get(snapshot_path)
        .cloned()
        .ok_or(Error::PasswordNotSet)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`{0}`")]
    Stronghold(#[from] iota_stronghold::Error),
    #[error("record not found")]
    RecordNotFound,
    #[error("failed to perform action: `{0}`")]
    FailedToPerformAction(String),
    #[error("snapshot password not set")]
    PasswordNotSet,
    #[error("invalid address or account index {0}")]
    TryFromInt(#[from] TryFromIntError),
    #[error("the mnemonic was already stored")]
    MnemonicAlreadyStored,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ActorRuntime {
    pub stronghold: Stronghold,
    spawned_client_paths: HashSet<Vec<u8>>,
    loaded_client_paths: HashSet<Vec<u8>>,
}

pub fn actor_runtime() -> &'static Arc<Mutex<ActorRuntime>> {
    static SYSTEM: Lazy<Arc<Mutex<ActorRuntime>>> = Lazy::new(|| {
        let system = SystemBuilder::new()
            .log(slog::Logger::root(slog::Discard, slog::o!()))
            .create()
            .unwrap();
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
async fn check_snapshot(
    runtime: &mut ActorRuntime,
    snapshot_path: &Path,
    password: Option<Arc<Password>>,
) -> Result<()> {
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
            switch_snapshot(runtime, snapshot_path, password).await?;
        } else if let Some(password) = password {
            if snapshot_path.exists() {
                stronghold_response_to_result(
                    runtime
                        .stronghold
                        .read_snapshot(
                            PRIVATE_DATA_CLIENT_PATH.to_vec(),
                            None,
                            &password.0,
                            None,
                            Some(snapshot_path.to_path_buf()),
                        )
                        .await,
                )?;
            }
        }
    } else {
        load_actors(runtime, snapshot_path, password).await?;
        CURRENT_SNAPSHOT_PATH
            .get_or_init(Default::default)
            .lock()
            .await
            .replace(snapshot_path.to_path_buf());
    }

    Ok(())
}

// saves the snapshot to the file system.
async fn save_snapshot(runtime: &mut ActorRuntime, snapshot_path: &Path) -> Result<()> {
    stronghold_response_to_result(
        runtime
            .stronghold
            .write_all_to_snapshot(
                &get_password(snapshot_path).await?.0,
                None,
                Some(snapshot_path.to_path_buf()),
            )
            .await,
    )
}

async fn clear_stronghold_cache(mut runtime: &mut ActorRuntime, persist: bool) -> Result<()> {
    if let Some(curr_snapshot_path) = CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .lock()
        .await
        .as_ref()
    {
        if persist && !runtime.spawned_client_paths.is_empty() {
            save_snapshot(runtime, curr_snapshot_path).await?;
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

async fn load_actors(runtime: &mut ActorRuntime, snapshot_path: &Path, password: Option<Arc<Password>>) -> Result<()> {
    // load all actors to prevent lost data on save
    load_private_data_actor(runtime, snapshot_path, password.clone()).await?;
    load_records_actor(runtime, snapshot_path, password).await?;
    Ok(())
}

async fn switch_snapshot(
    runtime: &mut ActorRuntime,
    snapshot_path: &Path,
    password: Option<Arc<Password>>,
) -> Result<()> {
    clear_stronghold_cache(runtime, true).await?;
    load_actors(runtime, snapshot_path, password).await?;

    CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .lock()
        .await
        .replace(snapshot_path.to_path_buf());

    Ok(())
}

async fn unset_password(storage_path: &Path) {
    let mut passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    let mut access_store = STRONGHOLD_ACCESS_STORE.get_or_init(Default::default).lock().await;
    access_store.remove(storage_path);
    passwords.remove(storage_path);
}

/// Removes the snapshot from memory and clears the password.
pub async fn unload_snapshot(storage_path: &Path, persist: bool) -> Result<()> {
    let current_snapshot_path = CURRENT_SNAPSHOT_PATH.get_or_init(Default::default).lock().await.clone();
    if let Some(current) = &current_snapshot_path {
        if current == storage_path {
            let mut runtime = actor_runtime().lock().await;
            clear_stronghold_cache(&mut runtime, persist).await?;
            CURRENT_SNAPSHOT_PATH.get_or_init(Default::default).lock().await.take();
        }
    }

    unset_password(storage_path).await;

    crate::event::emit_stronghold_status_change(&get_status(storage_path).await).await;

    Ok(())
}

pub async fn load_snapshot(snapshot_path: &Path, password: Vec<u8>) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    load_snapshot_internal(&mut runtime, snapshot_path, password).await
}

async fn load_snapshot_internal(runtime: &mut ActorRuntime, snapshot_path: &Path, password: Vec<u8>) -> Result<()> {
    if CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .lock()
        .await
        .as_deref()
        == Some(snapshot_path)
    {
        let (is_password_empty, is_password_updated) = {
            let passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
            let stored_password = passwords.get(snapshot_path).map(|p| &p.0);
            (stored_password.is_none(), stored_password != Some(&password))
        };
        if !runtime.spawned_client_paths.is_empty() && !is_password_empty && is_password_updated {
            save_snapshot(runtime, snapshot_path).await?;
        }
    }
    check_snapshot(runtime, snapshot_path, Some(Arc::new(Password(password.clone())))).await?;
    set_password(&snapshot_path, password).await;
    crate::event::emit_stronghold_status_change(&get_status(snapshot_path).await).await;
    Ok(())
}

/// Changes the snapshot password.
pub async fn change_password(snapshot_path: &Path, current_password: Vec<u8>, new_password: Vec<u8>) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    load_snapshot_internal(&mut runtime, snapshot_path, current_password).await?;

    stronghold_response_to_result(
        runtime
            .stronghold
            .write_all_to_snapshot(&new_password, None, Some(snapshot_path.to_path_buf()))
            .await,
    )?;

    set_password(snapshot_path, new_password).await;

    Ok(())
}

pub async fn store_mnemonic(snapshot_path: &Path, mnemonic: String) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path, None).await?;
    load_private_data_actor(&mut runtime, snapshot_path, None).await?;

    let mnemonic_location = Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH);
    if runtime.stronghold.record_exists(mnemonic_location.clone()).await {
        return Err(Error::MnemonicAlreadyStored);
    }

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::BIP39Recover {
            mnemonic,
            passphrase: None,
            output: mnemonic_location,
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

async fn derive(runtime: &mut ActorRuntime, chain: Chain) -> Result<Location> {
    let derive_output = Location::generic(SECRET_VAULT_PATH, DERIVE_OUTPUT_RECORD_PATH);
    let res = runtime
        .stronghold
        .runtime_exec(Procedure::SLIP10Derive {
            chain,
            input: SLIP10DeriveInput::Seed(Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH)),
            output: derive_output.clone(),
            hint: RecordHint::new("wallet.rs-derive").unwrap(),
        })
        .await;
    if let ProcResult::SLIP10Derive(response) = res {
        stronghold_response_to_result(response)?;
        Ok(derive_output)
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

async fn get_public_key(runtime: &mut ActorRuntime, derived_location: Location) -> Result<[u8; 32]> {
    let res = runtime
        .stronghold
        .runtime_exec(Procedure::Ed25519PublicKey {
            private_key: derived_location,
        })
        .await;
    if let ProcResult::Ed25519PublicKey(response) = res {
        stronghold_response_to_result(response)
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

pub async fn generate_address(
    snapshot_path: &Path,
    account_index: usize,
    address_index: usize,
    internal: bool,
) -> Result<Address> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path, None).await?;
    load_private_data_actor(&mut runtime, snapshot_path, None).await?;

    let chain = Chain::from_u32_hardened(vec![
        44,
        4218,
        account_index.try_into()?,
        internal as u32,
        address_index.try_into()?,
    ]);

    let derived_location = derive(&mut runtime, chain).await?;
    let public_key = get_public_key(&mut runtime, derived_location).await?;

    // Hash the public key to get the address
    let hash = Blake2b256::digest(&public_key);

    let ed25519_address = Ed25519Address::new(hash.try_into().unwrap());
    let address = Address::Ed25519(ed25519_address);
    Ok(address)
}

pub async fn sign_transaction(
    snapshot_path: &Path,
    message: &[u8],
    account_index: usize,
    address_index: usize,
    internal: bool,
) -> Result<Ed25519Signature> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path, None).await?;
    load_private_data_actor(&mut runtime, snapshot_path, None).await?;

    let chain = Chain::from_u32_hardened(vec![
        44,
        4218,
        account_index.try_into()?,
        internal as u32,
        address_index.try_into()?,
    ]);

    let derived_location = derive(&mut runtime, chain).await?;
    let public_key = get_public_key(&mut runtime, derived_location.clone()).await?;

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::Ed25519Sign {
            private_key: derived_location,
            msg: message.to_vec(),
        })
        .await;
    if let ProcResult::Ed25519Sign(response) = res {
        let signature = stronghold_response_to_result(response)?;
        Ok(Ed25519Signature::new(public_key, signature))
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

pub async fn get_record(snapshot_path: &Path, key: &str) -> Result<String> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path, None).await?;
    load_records_actor(&mut runtime, snapshot_path, None).await?;
    let (data, status) = runtime.stronghold.read_from_store(Location::generic(key, key)).await;
    stronghold_response_to_result(status).map_err(|_| Error::RecordNotFound)?;
    Ok(String::from_utf8_lossy(&data).to_string())
}

pub async fn store_record(snapshot_path: &Path, key: &str, record: String) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path, None).await?;

    // since we're creating a new account, we don't need to load it from the snapshot
    runtime.loaded_client_paths.insert(records_client_path());

    load_records_actor(&mut runtime, snapshot_path, None).await?;
    stronghold_response_to_result(
        runtime
            .stronghold
            .write_to_store(Location::generic(key, key), record.as_bytes().to_vec(), None)
            .await,
    )?;

    save_snapshot(&mut runtime, snapshot_path).await?;

    Ok(())
}

pub async fn remove_record(snapshot_path: &Path, key: &str) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    check_snapshot(&mut runtime, snapshot_path, None).await?;

    load_records_actor(&mut runtime, snapshot_path, None).await?;
    stronghold_response_to_result(runtime.stronghold.delete_from_store(Location::generic(key, key)).await)?;

    save_snapshot(&mut runtime, snapshot_path).await
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use rusty_fork::rusty_fork_test;
    use std::path::PathBuf;
    use tokio::time::Duration;

    rusty_fork_test! {
        #[test]
        fn password_expires() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let interval = 500;
                super::set_password_clear_interval(Duration::from_millis(interval)).await;
                let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).map(char::from).take(10).collect();
                std::fs::create_dir_all("./test-storage").unwrap();
                let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
                super::load_snapshot(&snapshot_path, [0; 32].to_vec()).await.unwrap();

                std::thread::sleep(Duration::from_millis(interval * 3));
                let res = super::get_record(&snapshot_path, "passwordexpires").await;
                assert!(res.is_err());
                let error = res.unwrap_err();
                if let super::Error::PasswordNotSet = error {
                    let status = super::get_status(&snapshot_path).await;
                    if let super::SnapshotStatus::Unlocked(_) = status.snapshot {
                        panic!("unexpected snapshot status");
                    }
                } else {
                    panic!("unexpected error: {:?}", error);
                }
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn action_keeps_password() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let interval = Duration::from_millis(900);
                super::set_password_clear_interval(interval).await;
                let snapshot_path: String = thread_rng().sample_iter(&Alphanumeric).map(char::from).take(10).collect();
                std::fs::create_dir_all("./test-storage").unwrap();
                let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
                super::load_snapshot(&snapshot_path, [0; 32].to_vec()).await.unwrap();

                for i in 1..6 {
                    let instant = std::time::Instant::now();
                    super::store_record(
                        &snapshot_path,
                        &format!("actionkeepspassword{}", i),
                        "data".to_string(),
                    )
                    .await
                    .unwrap();

                    let status = super::get_status(&snapshot_path).await;
                    if let super::SnapshotStatus::Locked = status.snapshot {
                        panic!("unexpected snapshot status");
                    }

                    if let Some(sleep_duration) = interval.checked_sub(instant.elapsed()) {
                        std::thread::sleep(sleep_duration / 2);
                    } else {
                        // if the elapsed > interval, set the password again
                        // this might happen if the test is stopped by another thread
                        super::set_password(&snapshot_path, [0; 32].to_vec()).await;
                    }
                }

                let id = "actionkeepspassword1".to_string();
                let res = super::get_record(&snapshot_path, &id).await;
                assert!(res.is_ok());

                std::thread::sleep(interval * 2);

                let res = super::get_record(&snapshot_path, &id).await;
                assert!(res.is_err());
                if let super::Error::PasswordNotSet = res.unwrap_err() {
                    let status = super::get_status(&snapshot_path).await;
                    if let super::SnapshotStatus::Unlocked(_) = status.snapshot {
                        panic!("unexpected snapshot status");
                    }
                } else {
                    panic!("unexpected error");
                }
            });
        }
    }

    #[tokio::test]
    async fn write_and_read() -> super::Result<()> {
        let snapshot_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(10)
            .collect();
        std::fs::create_dir_all("./test-storage").unwrap();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
        super::load_snapshot(&snapshot_path, [0; 32].to_vec()).await?;

        let id = "writeandreadtest".to_string();
        let data = "record data";
        super::store_record(&snapshot_path, &id, data.to_string()).await?;
        let stored_data = super::get_record(&snapshot_path, &id).await?;
        assert_eq!(stored_data, data);

        Ok(())
    }

    #[tokio::test]
    async fn write_and_delete() -> super::Result<()> {
        let snapshot_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(10)
            .collect();
        std::fs::create_dir_all("./test-storage").unwrap();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
        super::load_snapshot(&snapshot_path, [0; 32].to_vec()).await?;

        let id = "writeanddeleteid".to_string();
        let data = "record data";
        super::store_record(&snapshot_path, &id, data.to_string()).await?;
        super::remove_record(&snapshot_path, &id).await?;

        Ok(())
    }

    #[tokio::test]
    async fn write_and_read_multiple_snapshots() {
        let mut snapshot_saves = vec![];

        for i in 1..3 {
            let snapshot_path: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(10)
                .collect();
            std::fs::create_dir_all("./test-storage").unwrap();
            let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
            super::load_snapshot(&snapshot_path, [0; 32].to_vec()).await.unwrap();

            let id = format!("multiplesnapshots{}", i);
            let data: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(10)
                .collect();
            super::store_record(&snapshot_path, &id, data.clone())
                .await
                .expect("failed to store record");
            snapshot_saves.push((snapshot_path, id, data));
        }

        for (snapshot_path, key, data) in snapshot_saves {
            let stored_data = super::get_record(&snapshot_path, &key)
                .await
                .expect("failed to read record");
            assert_eq!(stored_data, data);
        }
    }

    #[tokio::test]
    async fn change_password() -> super::Result<()> {
        let snapshot_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(10)
            .collect();
        std::fs::create_dir_all("./test-storage").unwrap();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
        let old_password = [5; 32].to_vec();
        super::load_snapshot(&snapshot_path, old_password.to_vec()).await?;
        let id = "writeanddeleteid".to_string();
        let data = "record data";
        super::store_record(&snapshot_path, &id, data.to_string()).await?;

        let new_password = [6; 32].to_vec();
        super::change_password(&snapshot_path, old_password, new_password.to_vec()).await?;

        super::load_snapshot(&snapshot_path, new_password).await?;

        Ok(())
    }

    #[tokio::test]
    async fn change_password_invalid() -> super::Result<()> {
        let snapshot_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(10)
            .collect();
        std::fs::create_dir_all("./test-storage").unwrap();
        let snapshot_path = PathBuf::from(format!("./test-storage/{}.stronghold", snapshot_path));
        super::load_snapshot(&snapshot_path, [5; 32].to_vec()).await?;
        let id = "writeanddeleteid".to_string();
        let data = "record data";
        super::store_record(&snapshot_path, &id, data.to_string()).await?;

        let wrong_password = [16; 32].to_vec();
        let new_password = [6; 32].to_vec();
        match super::change_password(&snapshot_path, wrong_password, new_password.to_vec()).await {
            Err(super::Error::FailedToPerformAction(_)) => {}
            _ => panic!("expected a stronghold error when changing password"),
        }

        Ok(())
    }
}
