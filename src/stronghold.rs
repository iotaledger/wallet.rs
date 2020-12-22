// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Stronghold interface abstractions over an account

use crate::account::AccountIdentifier;
use iota::{Address, Ed25519Address, Ed25519Signature};
use iota_stronghold::{
    hd, Location, ProcResult, Procedure, RecordHint, ResultMessage, SLIP10DeriveInput, Stronghold, StrongholdFlags,
};
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

static PASSWORD_STORE: OnceCell<Arc<Mutex<HashMap<PathBuf, String>>>> = OnceCell::new();
static STRONGHOLD_ACCESS_STORE: OnceCell<Arc<Mutex<HashMap<PathBuf, bool>>>> = OnceCell::new();
static CURRENT_SNAPSHOT_PATH: OnceCell<Arc<Mutex<Option<PathBuf>>>> = OnceCell::new();
static PASSWORD_CLEAR_INTERVAL: OnceCell<Arc<Mutex<Duration>>> = OnceCell::new();
static PRIVATE_DATA_CLIENT_PATH: &[u8] = b"iota_seed";

const TIMEOUT: Duration = Duration::from_millis(5000);
const DEFAULT_PASSWORD_CLEAR_INTERVAL: Duration = Duration::from_secs(8 * 60);
const ACCOUNT_ID_SEPARATOR: char = '|';
const ACCOUNT_METADATA_VAULT_PATH: &str = "iota-wallet-account-metadata";
const ACCOUNT_IDS_RECORD_PATH: &str = "iota-wallet-account-ids";
const ACCOUNT_VAULT_PATH: &str = "iota-wallet-account-vault";
const ACCOUNT_RECORD_PATH: &str = "iota-wallet-account-record";
const SECRET_VAULT_PATH: &str = "iota-wallet-secret";
const SEED_RECORD_PATH: &str = "iota-wallet-seed";
const DERIVE_OUTPUT_RECORD_PATH: &str = "iota-wallet-derived";
const SNAPSHOT_FILENAME: &str = "wallet.stronghold";

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
    mut runtime: &mut ActorRuntime,
    snapshot_path: &PathBuf,
    client_path: Vec<u8>,
    flags: Vec<StrongholdFlags>,
) -> Result<()> {
    on_stronghold_access(&snapshot_path).await?;
    check_snapshot(&mut runtime, &snapshot_path).await?;

    if runtime.spawned_client_paths.contains(&client_path) {
        stronghold_response_to_result(runtime.stronghold.switch_actor_target(client_path))?;
    } else {
        stronghold_response_to_result(runtime.stronghold.spawn_stronghold_actor(client_path.clone(), flags))?;
        let snapshot_file_path = snapshot_path.join(SNAPSHOT_FILENAME);
        if snapshot_file_path.exists() {
            stronghold_response_to_result(
                runtime
                    .stronghold
                    .read_snapshot(
                        client_path.clone(),
                        None,
                        [0; 32].to_vec(), // TODO
                        None,
                        Some(snapshot_file_path),
                    )
                    .await,
            )?;
        }
        runtime.spawned_client_paths.insert(client_path);
    };

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

fn default_password_store() -> Arc<Mutex<HashMap<PathBuf, String>>> {
    thread::spawn(|| {
        crate::enter(|| {
            task::spawn(async {
                loop {
                    delay_for(
                        *PASSWORD_CLEAR_INTERVAL
                            .get_or_init(|| Arc::new(Mutex::new(DEFAULT_PASSWORD_CLEAR_INTERVAL)))
                            .lock()
                            .await,
                    )
                    .await;

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

pub async fn set_password<S: AsRef<Path>, P: Into<String>>(snapshot_path: S, password: P) {
    let mut passwords = PASSWORD_STORE.get_or_init(default_password_store).lock().await;
    passwords.insert(snapshot_path.as_ref().to_path_buf(), password.into());
}

async fn get_password<P: AsRef<Path>>(snapshot_path: P) -> Result<String> {
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

struct ActorRuntime {
    stronghold: Stronghold,
    spawned_client_paths: HashSet<Vec<u8>>,
}

fn system_runtime() -> &'static Arc<Mutex<ActorSystem>> {
    static SYSTEM: Lazy<Arc<Mutex<ActorSystem>>> = Lazy::new(|| {
        let system = ActorSystem::new().unwrap();
        Arc::new(Mutex::new(system))
    });
    &SYSTEM
}

fn actor_runtime() -> &'static Arc<Mutex<ActorRuntime>> {
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
        // save the current snapshot and read the new snapshot
        if curr_snapshot_path != snapshot_path {
            clear_stronghold_cache(&mut runtime).await?;
            let password = get_password(snapshot_path).await.unwrap();
            switch_snapshot(&mut runtime, snapshot_path, password).await?;
        }
    }

    Ok(())
}

async fn clear_stronghold_cache(runtime: &mut ActorRuntime) -> Result<()> {
    if let Some(curr_snapshot_path) = CURRENT_SNAPSHOT_PATH
        .get_or_init(Default::default)
        .lock()
        .await
        .as_ref()
    {
        runtime
            .stronghold
            .write_all_to_snapshot(
                [0; 32].to_vec(), // TODO use curr_snapshot_password
                None,
                Some(curr_snapshot_path.join(SNAPSHOT_FILENAME).to_path_buf()),
            )
            .await;
        for path in &runtime.spawned_client_paths {
            stronghold_response_to_result(runtime.stronghold.kill_stronghold(path.clone(), false).await)?;
            stronghold_response_to_result(runtime.stronghold.kill_stronghold(path.clone(), true).await)?;
        }
        runtime.spawned_client_paths = HashSet::new();
    }
    Ok(())
}

async fn switch_snapshot(mut runtime: &mut ActorRuntime, snapshot_path: &PathBuf, _password: String) -> Result<()> {
    clear_stronghold_cache(&mut runtime).await?;

    let mut current_snapshot_path = CURRENT_SNAPSHOT_PATH.get_or_init(Default::default).lock().await;
    current_snapshot_path.replace(snapshot_path.clone());

    Ok(())
}

pub async fn load_snapshot<P: Into<String>>(snapshot_path: &PathBuf, password: P) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
    let password = password.into();
    std::fs::create_dir_all(&snapshot_path).map_err(|_| Error::FailedToCreateSnapshotDir)?;
    set_password(&snapshot_path, password.clone()).await;
    check_snapshot(&mut runtime, &snapshot_path).await?;
    switch_snapshot(&mut runtime, &snapshot_path, password).await
}

pub async fn store_mnemonic(snapshot_path: &PathBuf, mnemonic: String) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;
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
        stronghold_response_to_result(status)
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
    load_private_data_actor(&mut runtime, snapshot_path).await?;

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::SLIP10Derive {
            chain: hd::Chain::from_u32_hardened(vec![
                44,
                4218,
                account_index.try_into()?,
                internal as u32,
                address_index.try_into()?,
            ]),
            input: SLIP10DeriveInput::Seed(Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH)),
            output: Location::generic(SECRET_VAULT_PATH, DERIVE_OUTPUT_RECORD_PATH),
            hint: RecordHint::new("wallet.rs-derived").unwrap(),
        })
        .await;
    if let ProcResult::SLIP10Derive(result) = res {
        let key: hd::Key = stronghold_response_to_result(result)?;
        Ok(Address::Ed25519(Ed25519Address::new(key.chain_code())))
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
    load_private_data_actor(&mut runtime, snapshot_path).await?;

    let res = runtime
        .stronghold
        .runtime_exec(Procedure::SignUnlockBlock {
            path: hd::Chain::from_u32_hardened(vec![
                44,
                4218,
                account_index.try_into()?,
                internal as u32,
                address_index.try_into()?,
            ]),
            key: Location::generic(SECRET_VAULT_PATH, SEED_RECORD_PATH),
            essence: transaction_essence,
        })
        .await;
    if let ProcResult::SignUnlockBlock(signature, public_key) = res {
        Ok(Ed25519Signature::new(
            stronghold_response_to_result(public_key)?,
            Box::new(stronghold_response_to_result(signature)?),
        ))
    } else {
        Err(Error::FailedToPerformAction(format!("{:?}", res)))
    }
}

pub async fn get_accounts(snapshot_path: &PathBuf) -> Result<Vec<String>> {
    let mut runtime = actor_runtime().lock().await;
    load_private_data_actor(&mut runtime, snapshot_path).await?;
    let account_ids_location = Location::generic(ACCOUNT_METADATA_VAULT_PATH, ACCOUNT_IDS_RECORD_PATH);
    let (data_opt, status) = runtime.stronghold.read_data(account_ids_location.clone()).await;

    let mut accounts = Vec::new();
    if let Some(data) = data_opt {
        let account_ids = String::from_utf8_lossy(&data).to_string();
        for account_id in account_ids.split(ACCOUNT_ID_SEPARATOR).filter(|id| id != &"") {
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
    data.filter(|data| data.len() > 0)
        .map(|data| String::from_utf8_lossy(&data).to_string())
        .ok_or(Error::AccountNotFound)
}

pub async fn get_account(snapshot_path: &PathBuf, account_id: &AccountIdentifier) -> Result<String> {
    let mut runtime = actor_runtime().lock().await;
    get_account_internal(&mut runtime, snapshot_path, account_id).await
}

pub async fn store_account(snapshot_path: &PathBuf, account_id: &AccountIdentifier, account: String) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;

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

    Ok(())
}

pub async fn remove_account(snapshot_path: &PathBuf, account_id: &AccountIdentifier) -> Result<()> {
    let mut runtime = actor_runtime().lock().await;

    // first we delete the account id from the reference array
    load_private_data_actor(&mut runtime, snapshot_path).await?;
    let account_ids_location = Location::generic(ACCOUNT_METADATA_VAULT_PATH, ACCOUNT_IDS_RECORD_PATH);
    let (data_opt, status) = runtime.stronghold.read_data(account_ids_location.clone()).await;
    let account_ids = data_opt
        .filter(|data| data.len() > 0)
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
    )
}

#[cfg(test)]
mod tests {
    use crate::account::AccountIdentifier;
    use rand::{thread_rng, Rng};
    use std::path::PathBuf;
    use tokio::time::Duration;

    #[tokio::test]
    async fn password_expires() -> super::Result<()> {
        let interval = 500;
        super::set_password_clear_interval(Duration::from_millis(interval)).await;
        let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
        let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, "password").await?;

        std::thread::sleep(Duration::from_millis(interval * 3));
        let res = super::get_account(&snapshot_path, &AccountIdentifier::Id("".to_string())).await;
        assert_eq!(res.is_err(), true);
        let error = res.unwrap_err();
        if let super::Error::PasswordNotSet = error {
        } else {
            panic!("unexpected error: {:?}", error);
        }

        Ok(())
    }

    #[tokio::test]
    async fn action_keeps_password() -> super::Result<()> {
        let interval = 500;
        super::set_password_clear_interval(Duration::from_millis(interval)).await;
        let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
        let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, "password").await?;

        for i in 1..5 {
            super::store_account(
                &snapshot_path,
                &AccountIdentifier::Id(i.to_string()),
                "data".to_string(),
            )
            .await?;
            std::thread::sleep(Duration::from_millis(interval / 2));
        }

        let id = AccountIdentifier::Id(1.to_string());
        let res = super::get_account(&snapshot_path, &id).await;
        assert_eq!(res.is_ok(), true);

        std::thread::sleep(Duration::from_millis(interval * 2));

        let res = super::get_account(&snapshot_path, &id).await;
        assert_eq!(res.is_err(), true);
        if let super::Error::PasswordNotSet = res.unwrap_err() {
        } else {
            panic!("unexpected error");
        }

        Ok(())
    }

    #[tokio::test]
    async fn write_and_read() -> super::Result<()> {
        let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
        let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, "password").await?;

        let id = AccountIdentifier::Id("id".to_string());
        let data = "account data";
        super::store_account(&snapshot_path, &id, data.to_string()).await?;
        let stored_data = super::get_account(&snapshot_path, &id).await?;
        assert_eq!(stored_data, data);

        Ok(())
    }

    #[tokio::test]
    async fn write_and_delete() -> super::Result<()> {
        let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
        let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
        super::load_snapshot(&snapshot_path, "password").await?;

        let id = AccountIdentifier::Id("id".to_string());
        let data = "account data";
        super::store_account(&snapshot_path, &id, data.to_string()).await?;
        super::remove_account(&snapshot_path, &id).await?;

        Ok(())
    }

    #[tokio::test]
    async fn write_and_read_multiple_snapshots() -> super::Result<()> {
        let mut snapshot_saves = vec![];

        for i in 1..3 {
            let snapshot_path: String = thread_rng().gen_ascii_chars().take(10).collect();
            let snapshot_path = PathBuf::from(format!("./example-database/{}", snapshot_path));
            super::load_snapshot(&snapshot_path, "password").await?;

            let id = AccountIdentifier::Id(i.to_string());
            let data: String = thread_rng().gen_ascii_chars().take(10).collect();
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
