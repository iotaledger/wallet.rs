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
    borrow::Cow,
    collections::HashMap,
    convert::TryInto,
    fs,
    num::NonZeroU64,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use futures::FutureExt;
use getset::Getters;
use iota::{MessageId, Payload};
use stronghold::Stronghold;
use tokio::{
    sync::{
        broadcast::{channel as broadcast_channel, Receiver as BroadcastReceiver, Sender as BroadcastSender},
        RwLock,
    },
    time::{delay_for, Duration as AsyncDuration},
};

/// The default storage path.
pub const DEFAULT_STORAGE_PATH: &str = "./example-database";

pub(crate) type AccountStore = Arc<RwLock<HashMap<AccountIdentifier, AccountHandle>>>;

/// Account manager builder.
pub struct AccountManagerBuilder {
    storage_path: PathBuf,
    default_storage: bool,
    polling_interval: Duration,
}

impl Default for AccountManagerBuilder {
    fn default() -> Self {
        Self {
            storage_path: DEFAULT_STORAGE_PATH.into(),
            default_storage: true,
            polling_interval: Duration::from_millis(30_000),
        }
    }
}

impl AccountManagerBuilder {
    /// Initialises a new instance of the account manager builder with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }

    /// Use the specified storage path when initialising the default storage adapter.
    pub fn with_storage_path(mut self, storage_path: impl AsRef<Path>) -> Self {
        self.storage_path = storage_path.as_ref().to_path_buf();
        self
    }

    /// Sets a custom storage adapter to be used.
    pub fn with_storage<S: StorageAdapter + Sync + Send + 'static>(
        mut self,
        storage_path: impl AsRef<Path>,
        adapter: S,
    ) -> Self {
        crate::storage::set_adapter(&storage_path, adapter);
        self.storage_path = storage_path.as_ref().to_path_buf();
        self.default_storage = false;
        self
    }

    /// Sets the polling interval.
    pub fn with_polling_interval(mut self, polling_interval: Duration) -> Self {
        self.polling_interval = polling_interval;
        self
    }

    /// Builds the manager.
    pub async fn finish(self) -> crate::Result<AccountManager> {
        if self.default_storage {
            crate::storage::set_adapter(
                &self.storage_path,
                crate::storage::get_adapter_from_path(&self.storage_path)?,
            );
        }

        let accounts = AccountManager::load_accounts(&self.storage_path)
            .await
            .unwrap_or_else(|_| Default::default());

        let mut instance = AccountManager {
            storage_path: self.storage_path,
            accounts,
            stop_polling_sender: None,
        };

        instance.start_background_sync(self.polling_interval).await;

        Ok(instance)
    }
}

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Getters)]
pub struct AccountManager {
    /// the path to the storage.
    #[getset(get = "pub")]
    storage_path: PathBuf,
    accounts: AccountStore,
    stop_polling_sender: Option<BroadcastSender<()>>,
}

impl Drop for AccountManager {
    fn drop(&mut self) {
        let accounts = self.accounts.clone();
        let stop_polling_sender = self.stop_polling_sender.clone();
        thread::spawn(move || {
            let _ = crate::block_on(async {
                for account in accounts.read().await.values() {
                    let _ = crate::monitor::unsubscribe(account.clone());
                }

                if let Some(sender) = stop_polling_sender {
                    sender.send(()).expect("failed to stop polling process");
                }
            });
        })
        .join()
        .expect("failed to stop monitoring and polling systems");
    }
}

impl AccountManager {
    /// Initialises the account manager builder.
    pub fn builder() -> AccountManagerBuilder {
        AccountManagerBuilder::new()
    }

    async fn load_accounts(storage_path: &PathBuf) -> crate::Result<AccountStore> {
        let accounts = crate::storage::with_adapter(&storage_path, |storage| storage.get_all())?;
        let accounts = crate::storage::parse_accounts(&storage_path, &accounts)?
            .into_iter()
            .map(|account| (account.id().clone(), account.into()))
            .collect();
        Ok(Arc::new(RwLock::new(accounts)))
    }

    /// Starts monitoring the accounts with the node's mqtt topics.
    async fn start_monitoring(&self) -> crate::Result<()> {
        for account in self.accounts.read().await.values() {
            crate::monitor::monitor_account_addresses_balance(account.clone()).await?;
            crate::monitor::monitor_unconfirmed_messages(account.clone()).await?;
        }
        Ok(())
    }

    /// Initialises the background polling and MQTT monitoring.
    async fn start_background_sync(&mut self, polling_interval: Duration) {
        let monitoring_disabled = self.start_monitoring().await.is_err();
        let (stop_polling_sender, stop_polling_receiver) = broadcast_channel(1);
        self.start_polling(polling_interval, monitoring_disabled, stop_polling_receiver);
        self.stop_polling_sender = Some(stop_polling_sender);
    }

    /// Sets the stronghold password.
    pub async fn set_stronghold_password<P: AsRef<str>>(&mut self, password: P) -> crate::Result<()> {
        let stronghold_path = self.storage_path.join(crate::storage::stronghold_snapshot_filename());
        let stronghold = Stronghold::new(
            &stronghold_path,
            !stronghold_path.exists(),
            password.as_ref().to_string(),
            None,
        )?;
        crate::init_stronghold(&self.storage_path, stronghold);

        if self.accounts.read().await.is_empty() {
            self.accounts = Self::load_accounts(&self.storage_path).await?;
            let _ = self.start_monitoring().await;
        }

        Ok(())
    }

    /// Starts the polling mechanism.
    fn start_polling(&self, polling_interval: Duration, is_monitoring_disabled: bool, mut stop: BroadcastReceiver<()>) {
        let storage_path = self.storage_path.clone();
        let accounts = self.accounts.clone();

        let interval = AsyncDuration::from_millis(polling_interval.as_millis().try_into().unwrap());

        thread::spawn(move || {
            crate::enter(|| {
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = async {
                                let storage_path_ = storage_path.clone();
                                let accounts_ = accounts.clone();

                                if let Err(error) = AssertUnwindSafe(poll(accounts_.clone(), storage_path_, is_monitoring_disabled))
                                    .catch_unwind()
                                    .await {
                                        if let Some(error) = error.downcast_ref::<crate::Error>() {
                                            // when the error is dropped, the on_error event will be triggered
                                        } else {
                                            let msg = if let Some(message) = error.downcast_ref::<Cow<'_, str>>() {
                                                format!("Internal error: {}", message)
                                            } else {
                                                "Internal error".to_string()
                                            };
                                            let _error = crate::Error::Panic(msg);
                                            // when the error is dropped, the on_error event will be triggered
                                        }
                                }

                                let accounts_ = accounts_.read().await;
                                for account_handle in accounts_.values() {
                                    let mut account = account_handle.write().await;
                                    let _ = account.save();
                                }

                                delay_for(interval).await;
                            } => {}
                            _ = stop.recv() => {}
                        }
                    }
                });
            });
        }).join().expect("failed to start polling");
    }

    /// Adds a new account.
    pub fn create_account(&self, client_options: ClientOptions) -> AccountInitialiser {
        AccountInitialiser::new(client_options, self.accounts.clone(), self.storage_path.clone())
    }

    /// Deletes an account.
    pub async fn remove_account(&self, account_id: &AccountIdentifier) -> crate::Result<()> {
        let mut accounts = self.accounts.write().await;

        {
            let account_handle = accounts.get(&account_id).ok_or(crate::Error::AccountNotFound)?;
            let account = account_handle.read().await;

            if !(account.messages().is_empty() && account.total_balance() == 0) {
                return Err(crate::Error::MessageNotEmpty);
            }
        }

        accounts.remove(account_id);

        if let Err(e) = crate::storage::with_adapter(&self.storage_path, |storage| storage.remove(&account_id)) {
            match e {
                // if we got an "AccountNotFound" error, that means we didn't save the cached account yet
                crate::Error::AccountNotFound => {}
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
        amount: NonZeroU64,
    ) -> crate::Result<Message> {
        let to_address = self
            .get_account(to_account_id)
            .await?
            .read()
            .await
            .latest_address()
            .ok_or(crate::Error::TransferDestinationEmpty)?
            .clone();

        let from_synchronized = self.get_account(from_account_id).await?.sync().await.execute().await?;
        from_synchronized
            .transfer(Transfer::builder(to_address.address().clone(), amount).finish())
            .await
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
            Err(crate::Error::StorageDoesntExist)
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
            return Err(crate::Error::AccountAlreadyImported {
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
                stronghold
                    .account_import(
                        Some(created_at_timestamp),
                        Some(created_at_timestamp),
                        stronghold_account.mnemonic().to_string(),
                        Some("password"),
                    )
                    .map_err(Into::into)
            });

            account.save()?;
        }
        crate::remove_stronghold(backup_stronghold_path);
        Ok(())
    }

    /// Gets the account associated with the given identifier.
    pub async fn get_account(&self, account_id: &AccountIdentifier) -> crate::Result<AccountHandle> {
        let accounts = self.accounts.read().await;
        accounts.get(account_id).cloned().ok_or(crate::Error::AccountNotFound)
    }

    /// Gets the account associated with the given alias (case insensitive).
    pub async fn get_account_by_alias<S: AsRef<str>>(&self, alias: S) -> Option<AccountHandle> {
        let alias = alias.as_ref().to_lowercase();
        for account_handle in self.accounts.read().await.values() {
            let account = account_handle.read().await;
            if account
                .alias()
                .to_lowercase()
                .chars()
                .zip(alias.chars())
                .all(|(x, y)| x == y)
            {
                return Some(account_handle.clone());
            }
        }
        None
    }

    /// Gets all accounts from storage.
    pub async fn get_accounts(&self) -> Vec<AccountHandle> {
        let accounts = self.accounts.read().await;
        accounts.values().cloned().collect()
    }

    /// Reattaches an unconfirmed transaction.
    pub async fn reattach(&self, account_id: &AccountIdentifier, message_id: &MessageId) -> crate::Result<Message> {
        let account = self.get_account(account_id).await?;
        account.sync().await.execute().await?.reattach(message_id).await
    }

    /// Promotes an unconfirmed transaction.
    pub async fn promote(&self, account_id: &AccountIdentifier, message_id: &MessageId) -> crate::Result<Message> {
        let account = self.get_account(account_id).await?;
        account.sync().await.execute().await?.promote(message_id).await
    }

    /// Retries an unconfirmed transaction.
    pub async fn retry(&self, account_id: &AccountIdentifier, message_id: &MessageId) -> crate::Result<Message> {
        let account = self.get_account(account_id).await?;
        account.sync().await.execute().await?.retry(message_id).await
    }
}

async fn poll(accounts: AccountStore, storage_path: PathBuf, syncing: bool) -> crate::Result<()> {
    let retried = if syncing {
        let mut accounts_before_sync = Vec::new();
        for account_handle in accounts.read().await.values() {
            accounts_before_sync.push(account_handle.read().await.clone());
        }
        let synced_accounts = sync_accounts(accounts.clone(), &storage_path, Some(0)).await?;
        let accounts_after_sync = accounts.read().await;

        // compare accounts to check for balance changes and new messages
        for account_before_sync in &accounts_before_sync {
            let account_after_sync = accounts_after_sync.get(account_before_sync.id()).unwrap();
            let account_after_sync = account_after_sync.read().await;

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
        for account_handle in accounts.read().await.values() {
            let (account_id, unconfirmed_messages): (AccountIdentifier, Vec<(MessageId, Payload)>) = {
                let account = account_handle.read().await;
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
                let new_message = repost_message(account_handle.clone(), &message_id, RepostAction::Retry).await?;
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
        let account = account_initialiser.initialise().await?;
        let synced_account = account.sync().await.execute().await?;
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
        let mut accounts = accounts.write().await;
        for account_handle in accounts.values_mut() {
            let mut sync = account_handle.sync().await;
            if let Some(index) = address_index {
                sync = sync.address_index(index);
            }
            let synced_account = sync.execute().await?;

            let account = account_handle.read().await;
            last_account = Some((
                account.messages().is_empty() || account.addresses().iter().all(|addr| *addr.balance() == 0),
                account.client_options().clone(),
                account.signer_type().clone(),
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
        let mut accounts = accounts.write().await;
        for (account_handle, synced_account) in discovered_accounts {
            accounts.insert(account_handle.id().await, account_handle);
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
        let account = synced.account_handle().read().await;

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
            let manager = manager.lock().unwrap();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            crate::block_on(async move {
                let account_handle = manager
                    .create_account(client_options)
                    .alias("alias")
                    .initialise()
                    .await
                    .expect("failed to add account");
                let account = account_handle.read().await;

                manager
                    .remove_account(account.id())
                    .await
                    .expect("failed to remove account");
            });
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

            crate::block_on(async move {
                let account_handle = manager
                    .create_account(client_options)
                    .messages(messages)
                    .initialise()
                    .await
                    .unwrap();

                let account = account_handle.read().await;
                let remove_response = manager.remove_account(account.id()).await;
                assert!(remove_response.is_err());
            });
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

            crate::block_on(async move {
                let account_handle = manager
                    .create_account(client_options)
                    .addresses(vec![AddressBuilder::new()
                        .balance(5)
                        .key_index(0)
                        .address(IotaAddress::Ed25519(Ed25519Address::new([0; 32])))
                        .outputs(vec![])
                        .build()
                        .unwrap()])
                    .initialise()
                    .await
                    .unwrap();
                let account = account_handle.read().await;

                let remove_response = manager.remove_account(account.id()).await;
                assert!(remove_response.is_err());
            });
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

            crate::block_on(async move {
                let account = manager
                    .create_account(client_options.clone())
                    .alias("alias")
                    .initialise()
                    .await
                    .expect("failed to add account");

                let create_response = manager.create_account(client_options).initialise().await;
                assert!(create_response.is_err());
            });
        }
    }
}
