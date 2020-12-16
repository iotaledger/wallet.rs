// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{
        account_id_to_stronghold_record_id, repost_message, AccountHandle, AccountIdentifier, AccountInitialiser,
        RepostAction, SyncedAccount,
    },
    client::ClientOptions,
    event::{emit_balance_change, emit_confirmation_state_change, emit_transaction_event, TransactionEventType},
    message::{Message, MessageType, Transfer},
    signing::SignerType,
    storage::StorageAdapter,
};

use std::{
    collections::HashMap,
    convert::TryInto,
    fs,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use futures::FutureExt;
use getset::{Getters, Setters};
use iota::{MessageId, Payload};
use stronghold::Stronghold;

/// The default storage path.
pub const DEFAULT_STORAGE_PATH: &str = "./example-database";

pub(crate) type AccountStore = Arc<RwLock<HashMap<AccountIdentifier, AccountHandle>>>;

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Getters, Setters)]
pub struct AccountManager {
    /// the path to the storage.
    #[getset(get = "pub")]
    storage_path: PathBuf,
    /// the polling interval.
    #[getset(get = "pub", set = "pub")]
    polling_interval: Duration,
    started_monitoring: bool,
    accounts: AccountStore,
    stop_polling: Arc<AtomicBool>,
    polling_handle: Option<JoinHandle<()>>,
}

impl Drop for AccountManager {
    fn drop(&mut self) {
        let _ = self.stop_background_sync();
    }
}

impl AccountManager {
    /// Initialises a new instance of the account manager with the default storage adapter.
    pub fn new() -> crate::Result<Self> {
        Self::with_storage_path(DEFAULT_STORAGE_PATH)
    }

    /// Initialises a new instance of the account manager with the default storage adapter using the specified storage
    /// path.
    pub fn with_storage_path(storage_path: impl AsRef<Path>) -> crate::Result<Self> {
        Self::with_storage_adapter(&storage_path, crate::storage::get_adapter_from_path(&storage_path)?)
    }

    /// Initialises a new instance of the account manager with the specified adapter.
    pub fn with_storage_adapter<S: StorageAdapter + Sync + Send + 'static>(
        storage_path: impl AsRef<Path>,
        adapter: S,
    ) -> crate::Result<Self> {
        crate::storage::set_adapter(&storage_path, adapter);
        let instance = Self {
            storage_path: storage_path.as_ref().to_path_buf(),
            polling_interval: Duration::from_millis(30_000),
            started_monitoring: false,
            accounts: Default::default(),
            stop_polling: Arc::new(AtomicBool::new(false)),
            polling_handle: None,
        };
        Ok(instance)
    }

    /// Loads the account from the storage into the manager instance.
    pub fn load_accounts(&mut self) -> crate::Result<()> {
        let empty_accounts = {
            let accounts = self.accounts.read().unwrap();
            accounts.is_empty()
        };
        if empty_accounts {
            let accounts = crate::storage::with_adapter(&self.storage_path, |storage| storage.get_all())?;
            let accounts = crate::storage::parse_accounts(&self.storage_path, &accounts)?
                .into_iter()
                .map(|account| (account.id().clone(), account.into()))
                .collect();
            self.accounts = Arc::new(RwLock::new(accounts));
        }
        Ok(())
    }

    /// Starts monitoring the accounts with the node's mqtt topics.
    fn start_monitoring(&self) -> crate::Result<()> {
        for account in self.accounts.read().unwrap().values() {
            crate::monitor::monitor_account_addresses_balance(account.clone())?;
            crate::monitor::monitor_unconfirmed_messages(account.clone())?;
        }
        Ok(())
    }

    /// Initialises the background polling and MQTT monitoring.
    pub fn start_background_sync(&mut self) {
        if !self.started_monitoring {
            let monitoring_disabled = self.start_monitoring().is_err();
            self.polling_handle = Some(self.start_polling(monitoring_disabled));
            self.started_monitoring = true;
        }
    }

    /// Stops the background polling and MQTT monitoring.
    pub fn stop_background_sync(&mut self) -> crate::Result<()> {
        {
            let accounts = self.accounts.read().unwrap();
            for account in accounts.values() {
                let _ = crate::monitor::unsubscribe(account.clone());
            }
        }

        if let Some(handle) = self.polling_handle.take() {
            self.stop_polling.store(true, Ordering::Relaxed);
            handle.join().expect("failed to stop polling thread");
        }

        Ok(())
    }

    /// Sets the stronghold password.
    pub fn set_stronghold_password<P: AsRef<str>>(&mut self, password: P) -> crate::Result<()> {
        let stronghold_path = self.storage_path.join(crate::storage::stronghold_snapshot_filename());
        let stronghold = Stronghold::new(
            &stronghold_path,
            !stronghold_path.exists(),
            password.as_ref().to_string(),
            None,
        )?;
        crate::init_stronghold(&self.storage_path, stronghold);
        self.start_background_sync();
        self.load_accounts()?;

        Ok(())
    }

    /// Starts the polling mechanism.
    fn start_polling(&self, is_monitoring_disabled: bool) -> JoinHandle<()> {
        let storage_path = self.storage_path.clone();
        let interval = self.polling_interval;
        let accounts = self.accounts.clone();
        let stop = self.stop_polling.clone();
        thread::spawn(move || {
            let sleep_duration = interval / 2;
            while !stop.load(Ordering::Relaxed) {
                let storage_path_ = storage_path.clone();
                let accounts_ = accounts.clone();
                crate::block_on(async move {
                    if let Err(panic) = AssertUnwindSafe(poll(accounts_, storage_path_, is_monitoring_disabled))
                        .catch_unwind()
                        .await
                    {
                        let msg = if let Some(message) = panic.downcast_ref::<String>() {
                            format!("Internal error: {}", message)
                        } else if let Some(message) = panic.downcast_ref::<&str>() {
                            format!("Internal error: {}", message)
                        } else {
                            "Internal error".to_string()
                        };
                        let _error = crate::WalletError::UnknownError(msg);
                        // when the error is dropped, the on_error event will be triggered
                    }
                });

                thread::sleep(sleep_duration);

                let accounts_ = accounts.read().unwrap();
                for account in accounts_.values() {
                    let mut account = account.write().unwrap();
                    let _ = account.save();
                }

                thread::sleep(sleep_duration);
            }
        })
    }

    /// Adds a new account.
    pub fn create_account(&self, client_options: ClientOptions) -> AccountInitialiser {
        AccountInitialiser::new(client_options, self.accounts.clone(), self.storage_path.clone())
    }

    /// Deletes an account.
    pub fn remove_account(&self, account_id: &AccountIdentifier) -> crate::Result<()> {
        let mut accounts = self.accounts.write().unwrap();

        {
            let account = accounts.get(&account_id).ok_or(crate::WalletError::AccountNotFound)?;
            let account = account.read().unwrap();

            if !(account.messages().is_empty() && account.total_balance() == 0) {
                return Err(crate::WalletError::MessageNotEmpty);
            }
        }

        accounts.remove(account_id);

        if let Err(e) = crate::storage::with_adapter(&self.storage_path, |storage| storage.remove(&account_id)) {
            match e {
                // if we got an "AccountNotFound" error, that means we didn't save the cached account yet
                crate::WalletError::AccountNotFound => {}
                _ => return Err(e),
            }
        }

        Ok(())
    }

    /// Syncs all accounts.
    pub async fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
        sync_accounts(self.accounts.clone(), &self.storage_path, None).await
    }

    /// Transfers an amount from an account to another.
    pub async fn internal_transfer(
        &self,
        from_account_id: &AccountIdentifier,
        to_account_id: &AccountIdentifier,
        amount: u64,
    ) -> crate::Result<Message> {
        let from_account_guard = self.get_account(from_account_id)?;
        let to_account_guard = self.get_account(to_account_id)?;

        let to_address = {
            let to_account = to_account_guard.read().unwrap();
            to_account
                .latest_address()
                .ok_or_else(|| anyhow::anyhow!("destination account address list empty"))?
                .clone()
        };

        let from_synchronized = from_account_guard.sync().execute().await?;
        let message = from_synchronized
            .transfer(Transfer::new(to_address.address().clone(), amount))
            .await?;

        Ok(message)
    }

    /// Backups the accounts to the given destination
    pub fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<PathBuf> {
        let storage_path = &self.storage_path;
        if storage_path.exists() {
            let metadata = fs::metadata(&storage_path)?;
            let backup_path = destination.as_ref().to_path_buf();
            if metadata.is_dir() {
                copy_dir(storage_path, &backup_path)?;
            } else {
                fs::create_dir_all(&destination)?;
                fs::copy(storage_path, &backup_path)?;
            }
            Ok(backup_path)
        } else {
            Err(crate::WalletError::StorageDoesntExist)
        }
    }

    /// Import backed up accounts.
    pub fn import_accounts<P: AsRef<Path>>(&self, source: P) -> crate::Result<()> {
        let backup_stronghold_path = source.as_ref().join(crate::storage::stronghold_snapshot_filename());
        let backup_stronghold =
            stronghold::Stronghold::new(&backup_stronghold_path, false, "password".to_string(), None)?;
        crate::init_stronghold(&source.as_ref().to_path_buf(), backup_stronghold);

        let backup_storage = crate::storage::get_adapter_from_path(&source)?;
        let accounts = backup_storage.get_all()?;
        let mut accounts = crate::storage::parse_accounts(&source.as_ref().to_path_buf(), &accounts)?;

        let stored_accounts = crate::storage::with_adapter(&self.storage_path, |storage| storage.get_all())?;
        let stored_accounts = crate::storage::parse_accounts(&self.storage_path, &stored_accounts)?;

        let already_imported_account = stored_accounts.iter().find(|stored_account| {
            stored_account.addresses().iter().any(|stored_address| {
                accounts.iter().any(|account| {
                    account
                        .addresses()
                        .iter()
                        .any(|address| address.address() == stored_address.address())
                })
            })
        });
        if let Some(imported_account) = already_imported_account {
            return Err(crate::WalletError::AccountAlreadyImported {
                alias: imported_account.alias().to_string(),
            });
        }

        let backup_stronghold =
            stronghold::Stronghold::new(&backup_stronghold_path, false, "password".to_string(), None)?;
        for account in accounts.iter_mut() {
            let stronghold_account =
                backup_stronghold.account_get_by_id(&account_id_to_stronghold_record_id(account.id())?)?;
            let created_at_timestamp: u128 = account.created_at().timestamp().try_into().unwrap(); // safe to unwrap since it's > 0
            let stronghold_account = crate::with_stronghold_from_path(&self.storage_path, |stronghold| {
                stronghold.account_import(
                    Some(created_at_timestamp),
                    Some(created_at_timestamp),
                    stronghold_account.mnemonic().to_string(),
                    Some("password"),
                )
            });

            account.save()?;
        }
        crate::remove_stronghold(backup_stronghold_path);
        Ok(())
    }

    /// Gets the account associated with the given identifier.
    pub fn get_account(&self, account_id: &AccountIdentifier) -> crate::Result<AccountHandle> {
        let accounts = self.accounts.read().unwrap();
        accounts
            .get(account_id)
            .cloned()
            .ok_or(crate::WalletError::AccountNotFound)
    }

    /// Gets the account associated with the given alias (case insensitive).
    pub fn get_account_by_alias<S: Into<String>>(&self, alias: S) -> Option<AccountHandle> {
        let alias = alias.into().to_lowercase();
        self.get_accounts().into_iter().find(|acc| {
            let acc = acc.read().unwrap();
            acc.alias().to_lowercase() == alias
        })
    }

    /// Gets all accounts from storage.
    pub fn get_accounts(&self) -> Vec<AccountHandle> {
        let accounts = self.accounts.read().unwrap();
        accounts.values().cloned().collect()
    }

    /// Reattaches an unconfirmed transaction.
    pub async fn reattach(&self, account_id: &AccountIdentifier, message_id: &MessageId) -> crate::Result<Message> {
        let account = self.get_account(account_id)?;
        account.sync().execute().await?.reattach(message_id).await
    }

    /// Promotes an unconfirmed transaction.
    pub async fn promote(&self, account_id: &AccountIdentifier, message_id: &MessageId) -> crate::Result<Message> {
        let account = self.get_account(account_id)?;
        account.sync().execute().await?.promote(message_id).await
    }

    /// Retries an unconfirmed transaction.
    pub async fn retry(&self, account_id: &AccountIdentifier, message_id: &MessageId) -> crate::Result<Message> {
        let account = self.get_account(account_id)?;
        account.sync().execute().await?.retry(message_id).await
    }
}

async fn poll(accounts: AccountStore, storage_path: PathBuf, syncing: bool) -> crate::Result<()> {
    let retried = if syncing {
        let mut accounts_before_sync = Vec::new();
        for account in accounts.read().unwrap().values() {
            accounts_before_sync.push(account.read().unwrap().clone());
        }
        let synced_accounts = sync_accounts(accounts.clone(), &storage_path, Some(0)).await?;
        let accounts_after_sync = accounts.read().unwrap();

        // compare accounts to check for balance changes and new messages
        for account_before_sync in &accounts_before_sync {
            let account_after_sync = accounts_after_sync
                .iter()
                .find(|(id, _)| id == &account_before_sync.id())
                .unwrap()
                .1;
            let account_after_sync = account_after_sync.read().unwrap();

            // balance event
            for address_before_sync in account_before_sync.addresses() {
                let address_after_sync = account_after_sync
                    .addresses()
                    .iter()
                    .find(|addr| addr == &address_before_sync)
                    .unwrap();
                if address_after_sync.balance() != address_before_sync.balance() {
                    emit_balance_change(
                        account_after_sync.id(),
                        address_after_sync,
                        *address_after_sync.balance(),
                    );
                }
            }

            // new messages event
            account_after_sync
                .messages()
                .iter()
                .filter(|message| !account_before_sync.messages().contains(message))
                .for_each(|message| {
                    emit_transaction_event(TransactionEventType::NewTransaction, account_after_sync.id(), &message)
                });

            // confirmation state change event
            for message in account_after_sync.messages() {
                let changed = match account_before_sync.messages().iter().find(|m| m.id() == message.id()) {
                    Some(old_message) => message.confirmed() != old_message.confirmed(),
                    None => false,
                };
                if changed {
                    emit_confirmation_state_change(account_after_sync.id(), &message, true);
                }
            }
        }
        retry_unconfirmed_transactions(synced_accounts).await?
    } else {
        let mut retried_messages = vec![];
        for account in accounts.read().unwrap().values() {
            let (account_id, unconfirmed_messages): (AccountIdentifier, Vec<(MessageId, Payload)>) = {
                let account = account.read().unwrap();
                let account_id = account.id().clone();
                let unconfirmed_messages = account
                    .list_messages(account.messages().len(), 0, Some(MessageType::Unconfirmed))
                    .iter()
                    .map(|m| (*m.id(), m.payload().clone()))
                    .collect();
                (account_id, unconfirmed_messages)
            };

            let mut promotions = vec![];
            let mut reattachments = vec![];
            for (message_id, payload) in unconfirmed_messages {
                let new_message = repost_message(account.clone(), &message_id, RepostAction::Retry).await?;
                if new_message.payload() == &payload {
                    reattachments.push(new_message);
                } else {
                    promotions.push(new_message);
                }
            }

            retried_messages.push(RetriedData {
                promoted: promotions,
                reattached: reattachments,
                account_id,
            });
        }

        retried_messages
    };

    retried.iter().for_each(|retried_data| {
        retried_data.reattached.iter().for_each(|message| {
            emit_transaction_event(TransactionEventType::Reattachment, &retried_data.account_id, &message);
        });
    });
    Ok(())
}

async fn discover_accounts(
    accounts: AccountStore,
    storage_path: &PathBuf,
    client_options: &ClientOptions,
    signer_type: Option<SignerType>,
) -> crate::Result<Vec<(AccountHandle, SyncedAccount)>> {
    let mut synced_accounts = vec![];
    loop {
        let mut account_initialiser =
            AccountInitialiser::new(client_options.clone(), accounts.clone(), storage_path.clone()).skip_persistance();
        if let Some(signer_type) = &signer_type {
            account_initialiser = account_initialiser.signer_type(signer_type.clone());
        }
        let account = account_initialiser.initialise()?;
        let synced_account = account.sync().execute().await?;
        let is_empty = *synced_account.is_empty();
        if is_empty {
            break;
        } else {
            synced_accounts.push((account, synced_account));
        }
    }
    Ok(synced_accounts)
}

async fn sync_accounts<'a>(
    accounts: AccountStore,
    storage_path: &PathBuf,
    address_index: Option<usize>,
) -> crate::Result<Vec<SyncedAccount>> {
    let mut synced_accounts = vec![];
    let mut last_account = None;

    {
        let mut accounts = accounts.write().unwrap();
        for account in accounts.values_mut() {
            let mut sync = account.sync();
            if let Some(index) = address_index {
                sync = sync.address_index(index);
            }
            let synced_account = sync.execute().await?;

            let account_ = account.read().unwrap();
            last_account = Some((
                account_.messages().is_empty() || account_.addresses().iter().all(|addr| *addr.balance() == 0),
                account_.client_options().clone(),
                account_.signer_type().clone(),
            ));
            synced_accounts.push(synced_account);
        }
    }

    let discovered_accounts_res = match last_account {
        Some((is_empty, client_options, signer_type)) => {
            if is_empty {
                discover_accounts(accounts.clone(), &storage_path, &client_options, Some(signer_type)).await
            } else {
                Ok(vec![])
            }
        }
        None => Ok(vec![]), /* None => discover_accounts(accounts.clone(), &storage_path, &ClientOptions::default(),
                             * None).await, */
    };

    if let Ok(discovered_accounts) = discovered_accounts_res {
        let mut accounts = accounts.write().unwrap();
        for (account, synced_account) in discovered_accounts {
            let account_id = {
                let account_ = account.read().unwrap();
                account_.id().clone()
            };
            accounts.insert(account_id, account);
            synced_accounts.push(synced_account);
        }
    }

    Ok(synced_accounts)
}

struct RetriedData {
    promoted: Vec<Message>,
    reattached: Vec<Message>,
    account_id: AccountIdentifier,
}

async fn retry_unconfirmed_transactions(synced_accounts: Vec<SyncedAccount>) -> crate::Result<Vec<RetriedData>> {
    let mut retried_messages = vec![];
    for synced in synced_accounts {
        let account = synced.account().read().unwrap();

        let unconfirmed_messages = account.list_messages(account.messages().len(), 0, Some(MessageType::Unconfirmed));
        let mut reattachments = vec![];
        let mut promotions = vec![];
        for message in unconfirmed_messages {
            let new_message = synced.retry(message.id()).await?;
            // if the payload is the same, it was reattached; otherwise it was promoted
            if new_message.payload() == message.payload() {
                reattachments.push(new_message);
            } else {
                promotions.push(new_message);
            }
        }
        retried_messages.push(RetriedData {
            promoted: promotions,
            reattached: reattachments,
            account_id: account.id().clone(),
        });
    }
    Ok(retried_messages)
}

fn copy_dir<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        let src: PathBuf = working_path.components().skip(input_root).collect();

        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name() {
                let dest_path = dest.join(filename);
                fs::copy(&path, &dest_path)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        address::{AddressBuilder, IotaAddress},
        client::ClientOptionsBuilder,
        message::Message,
    };
    use iota::{Ed25519Address, Indexation, MessageBuilder, MessageId, Payload};
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn store_accounts() {
            let manager = crate::test_utils::get_account_manager();
            let mut manager = manager.lock().unwrap();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let account = manager
                .create_account(client_options)
                .alias("alias")
                .initialise()
                .expect("failed to add account");
            let account_ = account.read().unwrap();

            manager.stop_background_sync().unwrap();

            manager
                .remove_account(account_.id())
                .expect("failed to remove account");
        }
    }

    rusty_fork_test! {
        #[test]
        fn remove_account_with_message_history() {
            let manager = crate::test_utils::get_account_manager();
            let manager = manager.lock().unwrap();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let messages = vec![Message::from_iota_message(
                MessageId::new([0; 32]),
                &[],
                &MessageBuilder::<crate::test_utils::NoopNonceProvider>::new()
                    .with_parent1(MessageId::new([0; 32]))
                    .with_parent2(MessageId::new([0; 32]))
                    .with_payload(Payload::Indexation(Box::new(
                        Indexation::new("index".to_string(), &[0; 16]).unwrap(),
                    )))
                    .with_network_id(0)
                    .with_nonce_provider(crate::test_utils::NoopNonceProvider {}, 0f64)
                    .finish()
                    .unwrap(),
                None,
            )
            .unwrap()];

            let account = manager
                .create_account(client_options)
                .messages(messages)
                .initialise()
                .unwrap();

            let account_ = account.read().unwrap();
            let remove_response = manager.remove_account(account_.id());
            assert!(remove_response.is_err());
        }
    }

    rusty_fork_test! {
        #[test]
        fn remove_account_with_balance() {
            let manager = crate::test_utils::get_account_manager();
            let manager = manager.lock().unwrap();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let account = manager
                .create_account(client_options)
                .addresses(vec![AddressBuilder::new()
                    .balance(5)
                    .key_index(0)
                    .address(IotaAddress::Ed25519(Ed25519Address::new([0; 32])))
                    .outputs(vec![])
                    .build()
                    .unwrap()])
                .initialise()
                .unwrap();
            let account_ = account.read().unwrap();

            let remove_response = manager.remove_account(account_.id());
            assert!(remove_response.is_err());
        }
    }

    rusty_fork_test! {
        #[test]
        fn create_account_with_latest_without_history() {
            let manager = crate::test_utils::get_account_manager();
            let manager = manager.lock().unwrap();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let account = manager
                .create_account(client_options.clone())
                .alias("alias")
                .initialise()
                .expect("failed to add account");

            let create_response = manager.create_account(client_options).initialise();
            assert!(create_response.is_err());
        }
    }
}
