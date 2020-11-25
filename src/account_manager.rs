use crate::account::{Account, AccountIdentifier, AccountInitialiser, SyncedAccount};
use crate::client::ClientOptions;
use crate::event::{
    emit_balance_change, emit_confirmation_state_change, emit_transaction_event,
    TransactionEventType,
};
use crate::message::{Message, MessageType, Transfer};
use crate::storage::StorageAdapter;

use std::convert::TryInto;
use std::fs;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use futures::FutureExt;
use iota::message::prelude::MessageId;
use stronghold::Stronghold;

const DEFAULT_STORAGE_PATH: &str = "./example-database";

/// The account manager.
///
/// Used to manage multiple accounts.
pub struct AccountManager {
    storage_path: PathBuf,
}

fn mutate_account_transaction<F: FnOnce(&Account, &mut Vec<Message>)>(
    storage_path: &PathBuf,
    account_id: AccountIdentifier,
    handler: F,
) -> crate::Result<()> {
    let mut account = crate::storage::get_account(&storage_path, account_id)?;
    let mut transactions: Vec<Message> = account.messages().to_vec();
    handler(&account, &mut transactions);
    account.set_messages(transactions);
    crate::storage::with_adapter(&storage_path, |storage| {
        storage.set(account_id, serde_json::to_string(&account)?)
    })?;
    Ok(())
}

impl AccountManager {
    /// Initialises a new instance of the account manager with the default storage adapter.
    pub fn new() -> crate::Result<Self> {
        Self::with_storage_path(DEFAULT_STORAGE_PATH)
    }

    /// Initialises a new instance of the account manager with the default storage adapter using the specified storage path.
    pub fn with_storage_path(storage_path: impl AsRef<Path>) -> crate::Result<Self> {
        Self::with_storage_adapter(
            &storage_path,
            crate::storage::get_adapter_from_path(&storage_path)?,
        )
    }

    /// Initialises a new instance of the account manager with the specified adapter.
    pub fn with_storage_adapter<S: StorageAdapter + Sync + Send + 'static>(
        storage_path: impl AsRef<Path>,
        adapter: S,
    ) -> crate::Result<Self> {
        crate::storage::set_adapter(&storage_path, adapter);
        let instance = Self {
            storage_path: storage_path.as_ref().to_path_buf(),
        };
        Ok(instance)
    }

    /// Sets the stronghold password.
    pub fn set_stronghold_password<P: AsRef<str>>(&self, password: P) -> crate::Result<()> {
        let stronghold_path = self
            .storage_path
            .join(crate::storage::stronghold_snapshot_filename());
        let stronghold = Stronghold::new(
            &stronghold_path,
            !stronghold_path.exists(),
            password.as_ref().to_string(),
            None,
        )?;
        crate::init_stronghold(&self.storage_path, stronghold);
        Ok(())
    }

    /// Enables syncing through node events.
    pub fn sync_through_events(&self) {
        let storage_path = self.storage_path.clone();
        // sync confirmation state changes
        crate::event::on_confirmation_state_change(move |event| {
            if *event.confirmed() {
                let _ = mutate_account_transaction(
                    &storage_path,
                    (**event.account_id()).into(),
                    |_, transactions| {
                        if let Some(message) = transactions
                            .iter_mut()
                            .find(|message| message.id() == event.message().id())
                        {
                            message.set_confirmed(true);
                        }
                    },
                );
            }
        });

        let storage_path = self.storage_path.clone();
        crate::event::on_broadcast(move |event| {
            let _ = mutate_account_transaction(
                &storage_path,
                event.account_id().clone().into(),
                |_, transactions| {
                    if let Some(message) = transactions
                        .iter_mut()
                        .find(|message| message.id() == event.message().id())
                    {
                        message.set_broadcasted(true);
                    }
                },
            );
        });

        let storage_path = self.storage_path.clone();
        crate::event::on_new_transaction(move |event| {
            let _ = mutate_account_transaction(
                &storage_path,
                event.account_id().clone().into(),
                |account, messages| {
                    messages.push((*event.message()).clone());
                },
            );
        });
    }

    /// Starts the polling mechanism.
    pub fn start_polling(&self, interval: Duration) -> thread::JoinHandle<()> {
        let storage_path = self.storage_path.clone();
        thread::spawn(move || {
            let mut runtime = tokio::runtime::Runtime::new().unwrap();
            loop {
                let storage_path_ = storage_path.clone();
                if crate::is_stronghold_initialised(&storage_path_) {
                    runtime.block_on(async move {
                        if let Err(panic) =
                            AssertUnwindSafe(poll(storage_path_)).catch_unwind().await
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
                }
                thread::sleep(interval);
            }
        })
    }

    /// Adds a new account.
    pub fn create_account(&self, client_options: ClientOptions) -> AccountInitialiser<'_> {
        AccountInitialiser::new(client_options, &self.storage_path)
    }

    /// Deletes an account.
    pub fn remove_account(&self, account_id: AccountIdentifier) -> crate::Result<()> {
        let account_str =
            crate::storage::with_adapter(&self.storage_path, |storage| storage.get(account_id))?;
        let account: Account = serde_json::from_str(&account_str)?;
        if !(account.messages().is_empty() && account.total_balance() == 0) {
            return Err(crate::WalletError::MessageNotEmpty);
        }
        crate::with_stronghold_from_path(&self.storage_path, |stronghold| {
            stronghold.account_remove(account.id())
        })?;
        crate::storage::with_adapter(&self.storage_path, |storage| storage.remove(account_id))?;
        Ok(())
    }

    /// Syncs all accounts.
    pub async fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
        sync_accounts(&self.storage_path, None).await
    }

    /// Transfers an amount from an account to another.
    pub async fn internal_transfer(
        &self,
        from_account_id: AccountIdentifier,
        to_account_id: AccountIdentifier,
        amount: u64,
    ) -> crate::Result<Message> {
        let mut from_account = self.get_account(from_account_id)?;
        let to_account = self.get_account(to_account_id)?;
        let to_address = to_account
            .latest_address()
            .ok_or_else(|| anyhow::anyhow!("destination account address list empty"))?
            .clone();
        let from_synchronized = from_account.sync().execute().await?;
        from_synchronized
            .transfer(Transfer::new(to_address.address().clone(), amount))
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
            Err(crate::WalletError::StorageDoesntExist)
        }
    }

    /// Import backed up accounts.
    pub fn import_accounts<P: AsRef<Path>>(&self, source: P) -> crate::Result<()> {
        let backup_stronghold_path = source
            .as_ref()
            .join(crate::storage::stronghold_snapshot_filename());
        let backup_stronghold = stronghold::Stronghold::new(
            &backup_stronghold_path,
            false,
            "password".to_string(),
            None,
        )?;
        crate::init_stronghold(&source.as_ref().to_path_buf(), backup_stronghold);

        let backup_storage = crate::storage::get_adapter_from_path(&source)?;
        let accounts = backup_storage.get_all()?;
        let accounts = crate::storage::parse_accounts(&source.as_ref().to_path_buf(), &accounts)?;

        let stored_accounts =
            crate::storage::with_adapter(&self.storage_path, |storage| storage.get_all())?;
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

        let backup_stronghold = stronghold::Stronghold::new(
            &backup_stronghold_path,
            false,
            "password".to_string(),
            None,
        )?;
        for account in accounts.iter() {
            let stronghold_account = backup_stronghold.account_get_by_id(account.id())?;
            let created_at_timestamp: u128 = account.created_at().timestamp().try_into().unwrap(); // safe to unwrap since it's > 0
            let stronghold_account =
                crate::with_stronghold_from_path(&self.storage_path, |stronghold| {
                    stronghold.account_import(
                        Some(created_at_timestamp),
                        Some(created_at_timestamp),
                        stronghold_account.mnemonic().to_string(),
                        Some("password"),
                    )
                });

            crate::storage::with_adapter(&self.storage_path, |storage| {
                storage.set(
                    account.id().clone().into(),
                    serde_json::to_string(&account)?,
                )
            })?;
        }
        crate::remove_stronghold(backup_stronghold_path);
        Ok(())
    }

    /// Gets the account associated with the given identifier.
    pub fn get_account(&self, account_id: AccountIdentifier) -> crate::Result<Account> {
        let mut account = crate::storage::get_account(&self.storage_path, account_id)?;
        account.set_storage_path(self.storage_path.clone());
        Ok(account)
    }

    /// Gets the account associated with the given alias (case insensitive).
    pub fn get_account_by_alias<S: Into<String>>(&self, alias: S) -> Option<Account> {
        let alias = alias.into().to_lowercase();
        if let Ok(accounts) = self.get_accounts() {
            accounts
                .into_iter()
                .find(|acc| acc.alias().to_lowercase() == alias)
        } else {
            None
        }
    }

    /// Gets all accounts from storage.
    pub fn get_accounts(&self) -> crate::Result<Vec<Account>> {
        crate::storage::with_adapter(&self.storage_path, |storage| {
            crate::storage::parse_accounts(&self.storage_path, &storage.get_all()?)
        })
    }

    /// Reattaches an unconfirmed transaction.
    pub async fn reattach(
        &self,
        account_id: AccountIdentifier,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        let mut account = self.get_account(account_id)?;
        account.sync().execute().await?.reattach(message_id).await
    }

    /// Promotes an unconfirmed transaction.
    pub async fn promote(
        &self,
        account_id: AccountIdentifier,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        let mut account = self.get_account(account_id)?;
        account.sync().execute().await?.promote(message_id).await
    }

    /// Retries an unconfirmed transaction.
    pub async fn retry(
        &self,
        account_id: AccountIdentifier,
        message_id: &MessageId,
    ) -> crate::Result<Message> {
        let mut account = self.get_account(account_id)?;
        account.sync().execute().await?.retry(message_id).await
    }
}

async fn poll(storage_path: PathBuf) -> crate::Result<()> {
    let accounts_before_sync =
        crate::storage::with_adapter(&storage_path, |storage| storage.get_all())?;
    let accounts_before_sync =
        crate::storage::parse_accounts(&storage_path, &accounts_before_sync)?;
    let synced_accounts = sync_accounts(&storage_path, Some(0)).await?;
    let accounts_after_sync =
        crate::storage::with_adapter(&storage_path, |storage| storage.get_all())?;
    let accounts_after_sync = crate::storage::parse_accounts(&storage_path, &accounts_after_sync)?;

    // compare accounts to check for balance changes and new messages
    for account_before_sync in &accounts_before_sync {
        let account_after_sync = accounts_after_sync
            .iter()
            .find(|account| account.id() == account_before_sync.id())
            .unwrap();

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
                emit_transaction_event(
                    TransactionEventType::NewTransaction,
                    account_after_sync.id(),
                    &message,
                )
            });

        // confirmation state change event
        account_after_sync.messages().iter().for_each(|message| {
            let changed = match account_before_sync
                .messages()
                .iter()
                .find(|m| m.id() == message.id())
            {
                Some(old_message) => message.confirmed() != old_message.confirmed(),
                None => false,
            };
            if changed {
                emit_confirmation_state_change(account_after_sync.id(), &message, true);
            }
        });
    }
    let retried = retry_unconfirmed_transactions(
        synced_accounts
            .iter()
            .zip(accounts_after_sync.iter())
            .collect(),
    )
    .await?;
    retried.iter().for_each(|retried_data| {
        retried_data.reattached.iter().for_each(|message| {
            emit_transaction_event(
                TransactionEventType::Reattachment,
                &retried_data.account_id,
                &message,
            );
        });
    });
    Ok(())
}

async fn discover_accounts(
    storage_path: &PathBuf,
    client_options: &ClientOptions,
) -> crate::Result<Vec<SyncedAccount>> {
    let mut synced_accounts = vec![];
    loop {
        let mut account = AccountInitialiser::new(client_options.clone(), &storage_path)
            .skip_persistance()
            .initialise()?;
        let synced_account = account.sync().skip_persistance().execute().await?;
        let is_empty = *synced_account.is_empty();
        if is_empty {
            break;
        } else {
            synced_accounts.push(synced_account);
            crate::storage::with_adapter(&storage_path, |storage| {
                storage.set(account.id().into(), serde_json::to_string(&account)?)
            })?;
        }
    }
    Ok(synced_accounts)
}

async fn sync_accounts<'a>(
    storage_path: &PathBuf,
    address_index: Option<usize>,
) -> crate::Result<Vec<SyncedAccount>> {
    let accounts = crate::storage::with_adapter(&storage_path, |storage| storage.get_all())?;
    let mut synced_accounts = vec![];
    let mut last_account = None;
    for account_str in accounts {
        let mut account: Account = serde_json::from_str(&account_str)?;
        account.set_storage_path(storage_path.clone());
        let mut sync = account.sync();
        if let Some(index) = address_index {
            sync = sync.address_index(index);
        }
        let synced_account = sync.execute().await?;
        last_account = Some(account);
        synced_accounts.push(synced_account);
    }

    let discovered_accounts = match last_account {
        Some(account) => {
            if account.messages().is_empty()
                || account.addresses().iter().all(|addr| *addr.balance() == 0)
            {
                discover_accounts(&storage_path, account.client_options()).await?
            } else {
                vec![]
            }
        }
        None => discover_accounts(&storage_path, &ClientOptions::default()).await?,
    };
    synced_accounts.extend(discovered_accounts.into_iter());

    Ok(synced_accounts)
}

struct RetriedData {
    promoted: Vec<Message>,
    reattached: Vec<Message>,
    account_id: [u8; 32],
}

async fn retry_unconfirmed_transactions(
    accounts: Vec<(&SyncedAccount, &Account)>,
) -> crate::Result<Vec<RetriedData>> {
    let mut retried_messages = vec![];
    for (synced, account) in accounts {
        let unconfirmed_messages =
            account.list_messages(account.messages().len(), 0, Some(MessageType::Unconfirmed));
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
            account_id: *account.id(),
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
    use crate::address::{AddressBuilder, IotaAddress};
    use crate::client::ClientOptionsBuilder;
    use crate::message::Message;
    use iota::message::prelude::{Ed25519Address, Indexation, MessageBuilder, MessageId, Payload};
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn store_accounts() {
            let manager = crate::test_utils::get_account_manager();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let account = manager
                .create_account(client_options)
                .alias("alias")
                .initialise()
                .expect("failed to add account");

            manager
                .remove_account(account.id().into())
                .expect("failed to remove account");
        }
    }

    rusty_fork_test! {
        #[test]
        fn remove_account_with_message_history() {
            let manager = crate::test_utils::get_account_manager();

            let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
                .expect("invalid node URL")
                .build();

            let account = manager
                .create_account(client_options)
                .messages(vec![Message::from_iota_message(MessageId::new([0; 32]), &[], &MessageBuilder::new()
                    .with_parent1(MessageId::new([0; 32]))
                    .with_parent2(MessageId::new([0; 32]))
                    .with_payload(Payload::Indexation(Box::new(Indexation::new(
                        "index".to_string(),
                        &[0; 16],
                    ).unwrap())))
                    .with_network_id(0)
                    .finish()
                    .unwrap()).unwrap()])
                .initialise().unwrap();

            let remove_response = manager.remove_account(account.id().into());
            assert!(remove_response.is_err());
        }
    }

    rusty_fork_test! {
        #[test]
        fn remove_account_with_balance() {
            let manager = crate::test_utils::get_account_manager();

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

            let remove_response = manager.remove_account(account.id().into());
            assert!(remove_response.is_err());
        }
    }

    rusty_fork_test! {
        #[test]
        fn create_account_with_latest_without_history() {
            let manager = crate::test_utils::get_account_manager();

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
