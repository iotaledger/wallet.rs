use crate::account::{Account, AccountIdentifier, AccountInitialiser, SyncedAccount};
use crate::client::ClientOptions;
use crate::storage::StorageAdapter;
use crate::transaction::{Transaction, TransactionType, Transfer};

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use iota::crypto::ternary::Hash;

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Default)]
pub struct AccountManager {}

fn mutate_account_transaction<F: FnOnce(&Account, &mut Vec<Transaction>)>(
    account_id: AccountIdentifier,
    handler: F,
) -> crate::Result<()> {
    let mut account = crate::storage::get_account(account_id.clone())?;
    let mut transactions: Vec<Transaction> = account.transactions().to_vec();
    handler(&account, &mut transactions);
    account.set_transactions(transactions);
    let adapter = crate::storage::get_adapter()?;
    adapter.set(account_id, serde_json::to_string(&account)?)?;
    Ok(())
}

impl AccountManager {
    /// Initialises a new instance of the account manager with the default storage adapter.
    pub fn new() -> Self {
        Default::default()
    }

    /// Enables syncing through node events.
    pub fn sync_through_events(&self) {
        // sync confirmation state changes
        crate::event::on_confirmation_state_change(|event| {
            if *event.confirmed() {
                let _ = mutate_account_transaction(
                    event.account_id().clone().into(),
                    |_, transactions| {
                        if let Some(tx) = transactions
                            .iter_mut()
                            .find(|tx| tx.hash() == event.transaction_hash())
                        {
                            tx.set_confirmed(true);
                        }
                    },
                );
            }
        });

        crate::event::on_broadcast(|event| {
            let _ =
                mutate_account_transaction(event.account_id().clone().into(), |_, transactions| {
                    if let Some(tx) = transactions
                        .iter_mut()
                        .find(|tx| tx.hash() == event.transaction_hash())
                    {
                        tx.set_broadcasted(true);
                    }
                });
        });

        crate::event::on_new_transaction(|event| {
            let transaction_hash = *event.transaction_hash();
            let _ = mutate_account_transaction(
                event.account_id().clone().into(),
                |account, transactions| {
                    let mut rt =
                        tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                    rt.block_on(async move {
                        let client = crate::client::get_client(account.client_options());
                        let response = client.get_trytes(&[transaction_hash]).await.unwrap();
                        let tx = response.trytes.first().unwrap().clone();
                        transactions.push(
                            Transaction::from_bundled(*event.transaction_hash(), tx).unwrap(),
                        );
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
        crate::storage::get_adapter()?.remove(account_id)?;
        // TODO remove seed from stronghold
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
    ) -> crate::Result<Transaction> {
        let from_account = self.get_account(from_account_id)?;
        let to_account = self.get_account(to_account_id)?;
        let from_synchronized = from_account.sync().execute().await?;
        from_synchronized
            .transfer(Transfer::new(to_account.latest_address().clone(), amount))
            .await
    }

    /// Backups the accounts to the given destination
    pub fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<PathBuf> {
        let storage_path = crate::storage::get_storage_path();
        if storage_path.exists() {
            let metadata = fs::metadata(&storage_path)?;
            let backup_path = destination.as_ref().join("backup");
            if metadata.is_dir() {
                copy_dir(storage_path, &backup_path)?;
            } else {
                fs::create_dir_all(destination)?;
                fs::copy(storage_path, &backup_path)?;
            }
            Ok(backup_path)
        } else {
            Err(anyhow::anyhow!("storage file doesn't exist"))
        }
    }

    /// Import backed up accounts.
    pub fn import_accounts<P: AsRef<Path>>(&self, source: P) -> crate::Result<()> {
        let storage = crate::storage::get_adapter()?;
        let backup_storage = crate::storage::get_adapter_from_path(source)?;

        let accounts = backup_storage.get_all()?;
        let accounts = crate::storage::parse_accounts(&accounts)?;

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

        for account in accounts {
            storage.set(
                account.id().clone().into(),
                serde_json::to_string(&account)?,
            )?;
        }
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
        transaction_hash: &Hash,
    ) -> crate::Result<()> {
        let mut account = self.get_account(account_id)?;
        reattach(&mut account, transaction_hash).await
    }
}

async fn sync_accounts() -> crate::Result<Vec<SyncedAccount>> {
    let accounts = crate::storage::get_adapter()?.get_all()?;
    let mut synced_accounts = vec![];
    for account_str in accounts {
        let account: Account = serde_json::from_str(&account_str)?;
        let synced_account = account.sync().execute().await?;
        synced_accounts.push(synced_account);
    }
    Ok(synced_accounts)
}

async fn reattach_unconfirmed_transactions() -> crate::Result<()> {
    let accounts = crate::storage::get_adapter()?.get_all()?;
    for account_str in accounts {
        let account: Account = serde_json::from_str(&account_str)?;
        let unconfirmed_transactions =
            account.list_transactions(1000, 0, Some(TransactionType::Unconfirmed));
        let mut account: Account = serde_json::from_str(&account_str)?;
        for tx in unconfirmed_transactions {
            reattach(&mut account, tx.hash()).await?;
        }
    }
    Ok(())
}

async fn reattach(account: &mut Account, transaction_hash: &Hash) -> crate::Result<()> {
    let mut transactions: Vec<Transaction> = account.transactions().to_vec();
    let transaction = transactions
        .iter_mut()
        .find(|tx| tx.hash() == transaction_hash)
        .ok_or_else(|| anyhow::anyhow!("transaction not found"))?;

    if transaction.confirmed {
        Err(anyhow::anyhow!("transaction is already confirmed"))
    } else if transaction.is_above_max_depth() {
        Err(anyhow::anyhow!("transaction is above max depth"))
    } else {
        let client = crate::client::get_client(account.client_options());
        let inclusion_states = client
            .get_inclusion_states()
            .transactions(&[*transaction.hash()])
            .send()
            .await?;
        if *inclusion_states.states.first().unwrap() {
            // transaction is already confirmed; do nothing
            transaction.set_confirmed(true);
        } else {
            // reattach the transaction
            let reattachment_transactions = client.reattach(transaction_hash).await?.send().await?;
            transactions.push(Transaction::from_bundled(
                *transaction_hash,
                reattachment_transactions.first().unwrap().clone(),
            )?);
        }
        // update the transactions in storage
        account.set_transactions(transactions);
        crate::storage::get_adapter()?.set(
            account.id().to_string().into(),
            serde_json::to_string(&account)?,
        )?;
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
    use super::AccountManager;
    use crate::account::Account;
    use crate::address::{Address, AddressBuilder};
    use crate::client::ClientOptionsBuilder;

    use iota::crypto::ternary::sponge::Kerl;
    use iota::signing::ternary::{
        seed::Seed,
        wots::{WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder},
        PrivateKey, PrivateKeyGenerator, PublicKey,
    };
    use iota::ternary::{T1B1Buf, TryteBuf};
    use iota::transaction::bundled::{Address as IotaAddress, BundledTransactionField};
    use rusty_fork::rusty_fork_test;

    fn _generate_iota_address(seed: &str) -> IotaAddress {
        let seed = Seed::from_trits(
            TryteBuf::try_from_str(seed)
                .unwrap()
                .as_trits()
                .encode::<T1B1Buf>(),
        )
        .unwrap();

        IotaAddress::try_from_inner(
            WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                .with_security_level(WotsSecurityLevel::Medium)
                .build()
                .unwrap()
                .generate_from_seed(&seed, 3)
                .unwrap()
                .generate_public_key()
                .unwrap()
                .as_trits()
                .to_owned(),
        )
        .unwrap()
    }

    fn _create_account(manager: &AccountManager, id: &str, addresses: Vec<Address>) -> Account {
        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        manager
            .create_account(client_options)
            .addresses(addresses)
            .alias(id)
            .id(id)
            .mnemonic(id)
            .initialise()
            .expect("failed to add account")
    }

    #[test]
    fn store_accounts() {
        let manager = AccountManager::new();
        let id = "test_store_accounts";
        _create_account(&manager, id, vec![]);

        manager
            .remove_account(id.to_string().into())
            .expect("failed to remove account");
    }

    fn _clear_db_and_backup(storage_path: &str) -> crate::Result<()> {
        let _ = std::fs::remove_dir_all(storage_path);
        #[cfg(feature = "sqlite")]
        crate::storage::set_storage_path(std::path::PathBuf::from(storage_path).join("wallet.db"))?;
        #[cfg(not(feature = "sqlite"))]
        crate::storage::set_storage_path(storage_path)?;
        let _ = std::fs::remove_dir_all("./backup");
        Ok(())
    }

    rusty_fork_test! {
        #[test]
        fn backup_and_restore_happy_path() {
            _clear_db_and_backup("./example-database/backup-test").unwrap();
            let manager = AccountManager::new();

            let id = "test_backup_and_restore";
            let account = _create_account(&manager, id, vec![]);

            // backup the stored accounts to ./backup/${backup_name}
            let backup_path = manager.backup("./backup").unwrap();

            // delete the account on the current storage
            manager.remove_account(id.to_string().into()).unwrap();

            // import the accounts from the backup and assert that it's the same
            manager.import_accounts(backup_path).unwrap();
            let imported_account = manager.get_account(id.to_string().into()).unwrap();
            assert_eq!(account, imported_account);
        }

        #[test]
        fn backup_and_restore_account_already_exists() {
            _clear_db_and_backup("./example-database/backup-test2").unwrap();
            let manager = AccountManager::new();

            // first we'll create an example account
            let id = "test_backup_and_restore_account_already_exists";
            let address = _generate_iota_address(
                "RVORZ9SIIP9RCYMREUIXXVPQIPHVCNPQ9HZWYKFWYWZRE9JQKG9REPKIASHUUECPSQO9JT9XNMVKWYGVA",
            );
            let address = AddressBuilder::new()
                .address(address.clone())
                .key_index(0)
                .balance(0)
                .build()
                .unwrap();
            let account = _create_account(&manager, id, vec![address]);

            let backup_path = manager.backup("./backup").unwrap();

            let response = manager.import_accounts(backup_path);
            assert!(response.is_err());
            let err = response.unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("Account {} already imported", account.alias())
            );
        }
    }
}
