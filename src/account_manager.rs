use crate::account::{Account, AccountIdentifier, AccountInitialiser, SyncedAccount};
use crate::client::ClientOptions;
use crate::message::{Message, MessageType, Transfer};
use crate::storage::StorageAdapter;

use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use iota::transaction::prelude::MessageId;
use stronghold::Stronghold;

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Default)]
pub struct AccountManager {}

fn mutate_account_transaction<F: FnOnce(&Account, &mut Vec<Message>)>(
    account_id: AccountIdentifier,
    handler: F,
) -> crate::Result<()> {
    let mut account = crate::storage::get_account(account_id.clone())?;
    let mut transactions: Vec<Message> = account.messages().to_vec();
    handler(&account, &mut transactions);
    account.set_messages(transactions);
    let adapter = crate::storage::get_adapter()?;
    adapter.set(account_id, serde_json::to_string(&account)?)?;
    Ok(())
}

impl AccountManager {
    /// Initialises a new instance of the account manager with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the stronghold password.
    pub fn set_stronghold_password<P: AsRef<str>>(&self, password: P) -> crate::Result<()> {
        let stronghold_path = crate::storage::get_stronghold_snapshot_path();
        let stronghold = Stronghold::new(
            &stronghold_path,
            !stronghold_path.exists(),
            password.as_ref().to_string(),
            None,
        )?;
        crate::init_stronghold(stronghold_path, stronghold);
        Ok(())
    }

    /// Enables syncing through node events.
    pub fn sync_through_events(&self) {
        // sync confirmation state changes
        crate::event::on_confirmation_state_change(|event| {
            if *event.confirmed() {
                let _ = mutate_account_transaction(
                    event.account_id().clone().into(),
                    |_, transactions| {
                        if let Some(message) = transactions
                            .iter_mut()
                            .find(|message| message.id() == event.message_id())
                        {
                            message.set_confirmed(true);
                        }
                    },
                );
            }
        });

        crate::event::on_broadcast(|event| {
            let _ =
                mutate_account_transaction(event.account_id().clone().into(), |_, transactions| {
                    if let Some(message) = transactions
                        .iter_mut()
                        .find(|message| message.id() == event.message_id())
                    {
                        message.set_broadcasted(true);
                    }
                });
        });

        crate::event::on_new_transaction(|event| {
            let message_id = *event.message_id();
            let _ = mutate_account_transaction(
                event.account_id().clone().into(),
                |account, messages| {
                    let mut rt =
                        tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                    rt.block_on(async move {
                        let client = crate::client::get_client(account.client_options());
                        let message = client.get_message(&message_id).data().unwrap();
                        messages.push(Message::from_iota_message(message_id, &message).unwrap());
                    });
                },
            );
        });
    }

    /// Starts the polling mechanism.
    pub fn start_polling(&self) {
        thread::spawn(move || async move {
            let _ = sync_accounts();
            let _ = reattach_unconfirmed_transactions();
            thread::sleep(Duration::from_secs(5));
        });
    }

    /// Adds a new account.
    pub fn create_account(&self, client_options: ClientOptions) -> AccountInitialiser {
        AccountInitialiser::new(client_options)
    }

    /// Deletes an account.
    pub fn remove_account(&self, account_id: AccountIdentifier) -> crate::Result<()> {
        let adapter = crate::storage::get_adapter()?;
        let account: Account = serde_json::from_str(&adapter.get(account_id.clone())?)?;
        if !(account.messages().is_empty() && account.total_balance() == 0) {
            return Err(anyhow::anyhow!(
                "can't delete an account with message history or balance"
            ));
        }
        crate::with_stronghold(|stronghold| stronghold.account_remove(account.id()))?;
        adapter.remove(account_id)?;
        Ok(())
    }

    /// Syncs all accounts.
    pub async fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
        sync_accounts().await
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
            .transfer(Transfer::new(to_address, amount))
            .await
    }

    /// Backups the accounts to the given destination
    pub fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<PathBuf> {
        let storage_path = crate::storage::get_storage_path();
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
            Err(anyhow::anyhow!("storage file doesn't exist"))
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
        crate::init_stronghold(backup_stronghold_path.clone(), backup_stronghold);

        let backup_storage = crate::storage::get_adapter_from_path(&source)?;
        let accounts = backup_storage.get_all()?;
        let accounts = crate::storage::parse_accounts(&accounts)?;

        let storage = crate::storage::get_adapter()?;
        let stored_accounts = storage.get_all()?;
        let stored_accounts = crate::storage::parse_accounts(&stored_accounts)?;

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
            return Err(anyhow::anyhow!(
                "Account {} already imported",
                imported_account.alias()
            ));
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
            let stronghold_account = crate::with_stronghold(|stronghold| {
                stronghold.account_import(
                    Some(created_at_timestamp),
                    Some(created_at_timestamp),
                    stronghold_account.mnemonic().to_string(),
                    Some("password"),
                )
            });

            storage.set(
                account.id().clone().into(),
                serde_json::to_string(&account)?,
            )?;
        }
        crate::remove_stronghold(backup_stronghold_path);
        Ok(())
    }

    /// Gets the account associated with the given identifier.
    pub fn get_account(&self, account_id: AccountIdentifier) -> crate::Result<Account> {
        crate::storage::get_account(account_id)
    }

    /// Reattaches an unconfirmed transaction.
    pub async fn reattach(
        &self,
        account_id: AccountIdentifier,
        message_id: &MessageId,
    ) -> crate::Result<()> {
        let mut account = self.get_account(account_id)?;
        reattach(&mut account, message_id).await
    }
}

async fn sync_accounts() -> crate::Result<Vec<SyncedAccount>> {
    let accounts = crate::storage::get_adapter()?.get_all()?;
    let mut synced_accounts = vec![];
    for account_str in accounts {
        let mut account: Account = serde_json::from_str(&account_str)?;
        let synced_account = account.sync().execute().await?;
        synced_accounts.push(synced_account);
    }
    Ok(synced_accounts)
}

async fn reattach_unconfirmed_transactions() -> crate::Result<()> {
    let accounts = crate::storage::get_adapter()?.get_all()?;
    for account_str in accounts {
        let account: Account = serde_json::from_str(&account_str)?;
        let unconfirmed_messages = account.list_messages(1000, 0, Some(MessageType::Unconfirmed));
        let mut account: Account = serde_json::from_str(&account_str)?;
        for message in unconfirmed_messages {
            reattach(&mut account, &message.id()).await?;
        }
    }
    Ok(())
}

async fn reattach(account: &mut Account, message_id: &MessageId) -> crate::Result<()> {
    let mut messages: Vec<Message> = account.messages().to_vec();
    let message = messages
        .iter_mut()
        .find(|message| message.id() == message_id)
        .ok_or_else(|| anyhow::anyhow!("message not found"))?;

    if message.confirmed {
        Err(anyhow::anyhow!("message is already confirmed"))
    } else if message.is_above_max_depth() {
        Err(anyhow::anyhow!("message is above max depth"))
    } else {
        let client = crate::client::get_client(account.client_options());
        if *client
            .is_confirmed(&[*message_id])?
            .get(message.id())
            .ok_or_else(|| anyhow::anyhow!("invalid `is_confirmed` response"))?
        {
            // message is already confirmed; do nothing
            message.set_confirmed(true);
        } else {
            // reattach the message
            let reattachment_message = client.reattach(&message_id)?;
            messages.push(Message::from_iota_message(
                *message_id,
                &reattachment_message,
            )?);
        }
        // update the messages in storage
        account.set_messages(messages);
        crate::storage::get_adapter()?
            .set(account.id().into(), serde_json::to_string(&account)?)?;
        Ok(())
    }
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
    use iota::transaction::prelude::{
        Ed25519Address, Indexation, MessageBuilder, MessageId, Payload,
    };
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
                .messages(vec![Message::from_iota_message(MessageId::new([0; 32]), &MessageBuilder::new()
                    .parent1(MessageId::new([0; 32]))
                    .parent2(MessageId::new([0; 32]))
                    .payload(Payload::Indexation(Box::new(Indexation::new(
                        "".to_string(),
                        Box::new([0; 16]),
                    ))))
                    .build()
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
