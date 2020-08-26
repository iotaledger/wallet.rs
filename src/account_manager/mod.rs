mod api;

use crate::account::{Account, AccountIdentifier, AccountInitialiser};
use crate::client::ClientOptions;
use crate::transaction::Transaction;
use api::{AccountSynchronizer, SyncedAccount};
use std::path::Path;
use std::thread;
use std::time::Duration;

/// The account manager.
///
/// Used to manage multiple accounts.
#[derive(Default)]
pub struct AccountManager {}

fn mutate_account_transaction<F: FnOnce(&Account<'_>, &mut Vec<Transaction>)>(
    account_id: AccountIdentifier,
    handler: F,
) -> crate::Result<()> {
    let mut account = get_account(account_id.clone())?;
    let mut transactions: Vec<Transaction> = account.transactions().iter().cloned().collect();
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
            let transaction_hash = event.transaction_hash().clone();
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
        thread::spawn(move || {
            let _ = sync_accounts();
            thread::sleep(Duration::from_secs(5));
        });
    }

    /// Adds a new account.
    pub fn create_account<'a>(&self, client_options: ClientOptions) -> AccountInitialiser<'a> {
        AccountInitialiser::new(client_options)
    }

    /// Deletes an account.
    pub fn remove_account(&self, account_id: AccountIdentifier) -> crate::Result<()> {
        crate::storage::get_adapter()?.remove(account_id)
    }

    /// Syncs all accounts.
    pub async fn sync_accounts(&self) -> crate::Result<Vec<SyncedAccount>> {
        sync_accounts().await
    }

    /// Transfers an amount from an account to another.
    pub fn transfer(
        &self,
        from_account_id: AccountIdentifier,
        to_account_id: AccountIdentifier,
        amount: u64,
    ) -> crate::Result<()> {
        unimplemented!()
    }

    /// Backups the accounts to the given destination
    pub fn backup<P: AsRef<Path>>(&self, destination: P) -> crate::Result<()> {
        unimplemented!()
    }

    /// Import backed up accounts.
    pub fn import_accounts<'a>(&self, accounts: Vec<Account<'a>>) -> crate::Result<()> {
        unimplemented!()
    }

    /// Gets the account associated with the given identifier.
    pub fn get_account<'a>(&self, account_id: AccountIdentifier) -> crate::Result<Account<'a>> {
        get_account(account_id)
    }

    /// Reattaches an unconfirmed transaction.
    pub fn reattach<T>(&self, account_id: AccountIdentifier) -> crate::Result<()> {
        unimplemented!()
    }
}

async fn sync_accounts() -> crate::Result<Vec<SyncedAccount>> {
    let accounts = crate::storage::get_adapter()?.get_all()?;
    let mut synced_accounts = vec![];
    for account_str in accounts {
        let account: Account<'_> = serde_json::from_str(&account_str)?;
        let synced_account = AccountSynchronizer::new(&account).execute().await?;
        synced_accounts.push(synced_account);
    }
    Ok(synced_accounts)
}

fn get_account<'a>(account_id: AccountIdentifier) -> crate::Result<Account<'a>> {
    let account_str = crate::storage::get_adapter()?.get(account_id)?;
    // serde_json::from_str(&account_str).map_err(|e| e.into());
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::AccountManager;
    use crate::client::ClientOptionsBuilder;

    #[test]
    fn store_accounts() {
        let manager = AccountManager::new();
        let id = "test";
        let client_options = ClientOptionsBuilder::node("https://nodes.devnet.iota.org:443")
            .expect("invalid node URL")
            .build();

        manager
            .create_account(client_options)
            .alias(id)
            .id(id)
            .mnemonic(id)
            // TODO .nodes(vec!["https://nodes.devnet.iota.org:443"])
            .initialise()
            .expect("failed to add account");

        manager
            .remove_account(id.to_string().into())
            .expect("failed to remove account");
    }
}
