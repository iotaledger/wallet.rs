use crate::account::{Account, AccountIdentifier};
use crate::address::{Address, AddressBuilder, IotaAddress};
use crate::client::get_client;
use crate::transaction::{Transaction, Transfer};

use iota::crypto::ternary::Hash;
use iota::transaction::{
    bundled::{BundledTransactionBuilder, BundledTransactionField, Value as IotaValue},
    Vertex,
};

use std::convert::TryInto;

mod input_selection;

/// Syncs addresses with the tangle.
/// The method ensures that the wallet local state has all used addresses plus an unused address.
///
/// To sync addresses for an account from scratch, `address_index` = 0 and `gap_limit` = 20 should be provided.
/// To sync addresses from the latest address, `address_index` = latest address index and `gap_limit` = 1 should be provided.
///
/// # Arguments
///
/// * `address_index` The address index.
/// * `gap_limit` Number of addresses indexes that are generated.
///
/// # Return value
///
/// Returns a (addresses, hashes) tuples representing the address history up to latest unused address,
/// and the transaction hashes associated with the addresses.
///
async fn sync_addresses(
    account: &'_ Account,
    address_index: usize,
    gap_limit: Option<usize>,
) -> crate::Result<(Vec<Address>, Vec<Hash>)> {
    let mut address_index = address_index;
    let account_addresses = account.addresses();
    let account_transactions = account.transactions();
    let account_latest_address = account.latest_address();

    let client = get_client(account.client_options());
    let gap_limit = gap_limit.unwrap_or(20);

    let mut generated_addresses = vec![];
    let mut found_transactions = vec![];
    loop {
        let mut generated_iota_addresses = vec![];
        for i in address_index..gap_limit {
            let existing_address = account_addresses.iter().find(|a| *a.key_index() == i);
            let (_, iota_address) = match existing_address {
                Some(address) => (
                    account_addresses.iter().position(|a| a == address).unwrap(),
                    address.address().clone(),
                ),
                None => crate::address::get_new_iota_address(&account)?,
            };
            generated_iota_addresses.push(iota_address);
        }

        let curr_found_transactions = client
            .find_transactions()
            .addresses(&generated_iota_addresses[..])
            .send()
            .await?
            .hashes;
        found_transactions.extend(curr_found_transactions.iter().cloned());

        let balances = client
            .get_balances()
            .addresses(&generated_iota_addresses[..])
            .send()
            .await?
            .balances;
        let mut balances_iter = balances.iter();

        for iota_address in generated_iota_addresses {
            let balance = balances_iter
                .next()
                .ok_or_else(|| anyhow::anyhow!("Address balance not found"))?;
            let address = AddressBuilder::new()
                .address(iota_address)
                .key_index(address_index)
                .balance(*balance)
                .build()?;
            generated_addresses.push(address);
            address_index += 1;
        }

        if found_transactions.len() == 0 && balances.iter().all(|b| *b == 0) {
            break;
        }
    }

    Ok((generated_addresses, found_transactions))
}

/// Syncs transactions with the tangle.
/// The method should ensures that the wallet local state has transactions associated with the address history.
async fn sync_transactions<'a>(
    account: &'a Account,
    new_transaction_hashes: Vec<Hash>,
) -> crate::Result<Vec<Transaction>> {
    let mut transactions: Vec<Transaction> = account.transactions().to_vec();

    // sync `broadcasted` state
    transactions
        .iter_mut()
        .filter(|tx| !tx.broadcasted() && new_transaction_hashes.contains(tx.hash()))
        .for_each(|tx| {
            tx.set_broadcasted(true);
        });

    // sync `confirmed` state
    let mut unconfirmed_transactions: Vec<&mut Transaction> = transactions
        .iter_mut()
        .filter(|tx| !tx.confirmed())
        .collect();
    let client = get_client(account.client_options());
    let unconfirmed_transaction_hashes: Vec<Hash> = unconfirmed_transactions
        .iter()
        .map(|tx| *tx.hash())
        .collect();
    let confirmed_states = client
        .is_confirmed(&unconfirmed_transaction_hashes[..])
        .await
        .unwrap();
    for (tx, confirmed) in unconfirmed_transactions
        .iter_mut()
        .zip(confirmed_states.iter())
    {
        if *confirmed {
            tx.set_confirmed(true);
        }
    }

    // get new transactions
    let response = client
        .get_trytes(&new_transaction_hashes[..])
        .await
        .unwrap();
    let mut hashes_iter = new_transaction_hashes.iter();

    for tx in response.trytes {
        let hash = hashes_iter.next().unwrap();
        transactions.push(Transaction::from_bundled(*hash, tx).unwrap());
    }

    Ok(transactions)
}

/// Account sync helper.
pub struct AccountSynchronizer<'a> {
    account: &'a Account,
    address_index: usize,
    gap_limit: Option<usize>,
    skip_persistance: bool,
}

impl<'a> AccountSynchronizer<'a> {
    /// Initialises a new instance of the sync helper.
    pub(super) fn new(account: &'a Account) -> Self {
        Self {
            account,
            address_index: account.addresses().len(),
            gap_limit: None,
            skip_persistance: false,
        }
    }

    /// Number of address indexes that are generated.
    pub fn gap_limit(mut self, limit: usize) -> Self {
        self.gap_limit = Some(limit);
        self
    }

    /// Skip write to the database.
    pub fn skip_persistance(mut self) -> Self {
        self.skip_persistance = true;
        self
    }

    /// Syncs account with the tangle.
    /// The account syncing process ensures that the latest metadata (balance, transactions)
    /// associated with an account is fetched from the tangle and is stored locally.
    pub async fn execute(self) -> crate::Result<SyncedAccount> {
        let client = get_client(self.account.client_options());
        let mut addresses = vec![];
        for address in self.account.addresses() {
            addresses.push(address.address().clone());
        }

        let mut new_transaction_hashes = vec![];
        let find_transactions_response = client
            .find_transactions()
            .addresses(&addresses[..])
            .send()
            .await
            .unwrap();

        for tx_hash in find_transactions_response.hashes {
            if !self
                .account
                .transactions()
                .iter()
                .any(|tx| tx.hash() == &tx_hash)
            {
                new_transaction_hashes.push(tx_hash);
            }
        }

        sync_addresses(self.account, self.address_index, self.gap_limit).await?;
        sync_transactions(self.account, new_transaction_hashes).await?;

        let synced_account = SyncedAccount {
            account_id: *self.account.id(),
            deposit_address: AddressBuilder::new()
                .address(IotaAddress::zeros())
                .balance(0)
                .key_index(0)
                .build()?,
        };
        Ok(synced_account)
    }
}

/// Data returned from account synchronization.
pub struct SyncedAccount {
    account_id: [u8; 32],
    deposit_address: Address,
}

impl SyncedAccount {
    /// The account's deposit address.
    pub fn deposit_address(&self) -> &Address {
        &self.deposit_address
    }

    /// Selects input addresses for a value transaction.
    /// The method ensures that the recipient address doesn’t match any of the selected inputs or the remainder address.
    ///
    /// # Arguments
    ///
    /// * `threshold` Amount user wants to spend.
    /// * `address` Recipient address.
    ///
    /// # Return value
    ///
    /// Returns a (addresses, address) tuple representing the selected input addresses and the remainder address if needed.
    fn select_inputs(
        &self,
        threshold: u64,
        address: &Address,
    ) -> crate::Result<(Vec<Address>, Option<Address>)> {
        // TODO
        let inputs = input_selection::select_input(threshold, &mut vec![])?;
        unimplemented!()
    }

    /// Send transactions.
    pub async fn transfer(&self, transfer_obj: Transfer) -> crate::Result<Transaction> {
        if *transfer_obj.amount() == 0 {
            return Err(anyhow::anyhow!("amount can't be zero"));
        }
        if transfer_obj.address().checksum()
            != &crate::address::generate_checksum(transfer_obj.address().address())?
        {
            return Err(anyhow::anyhow!("invalid address checksum"));
        }
        let value = (*transfer_obj.amount()).try_into()?;
        let account_id: AccountIdentifier = self.account_id.clone().into();

        let adapter = crate::storage::get_adapter()?;

        let mut account = crate::storage::get_account(account_id.clone())?;
        let client = get_client(account.client_options());

        let (inputs, remainder) =
            self.select_inputs(*transfer_obj.amount(), transfer_obj.address())?;
        let transactions_to_approve = client.get_transactions_to_approve().send().await?;

        let transaction = BundledTransactionBuilder::new()
            .with_address(transfer_obj.address().address().clone())
            .with_value(IotaValue::try_from_inner(value).unwrap()) // TODO error handling
            .with_trunk(transactions_to_approve.trunk_transaction)
            .with_branch(transactions_to_approve.branch_transaction)
            .build()
            .unwrap(); // TODO error handling

        let attached = client
            .attach_to_tangle()
            .trunk_transaction(transaction.trunk())
            .branch_transaction(transaction.branch())
            .trytes(&[transaction])
            .send()
            .await?;

        let transactions = client.send_trytes().trytes(attached.trytes).send().await?;
        let transactions: Vec<Transaction> = transactions
            .iter()
            .map(|tx| Transaction::from_bundled(*tx.bundle(), tx.clone()).unwrap())
            .collect();
        let tx = transactions.first().unwrap().clone();
        account.append_transactions(transactions);
        adapter.set(account_id, serde_json::to_string(&account)?)?;

        Ok(tx)
    }

    /// Retry transactions.
    pub fn retry(&self, transaction_hash: &Hash) -> crate::Result<Transaction> {
        let account: Account = crate::storage::get_account(self.account_id.clone().into())?;
        let transaction = account
            .get_transaction(transaction_hash)
            .ok_or_else(|| anyhow::anyhow!("transaction with the given hash not found"));
        unimplemented!()
    }
}
