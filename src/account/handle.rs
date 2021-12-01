// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::account::{
    operations::{
        address_generation,
        address_generation::AddressGenerationOptions,
        balance_finder::search_addresses_with_funds,
        syncing::{sync_account, SyncOptions},
        transfer::{send_transfer, TransferOptions, TransferOutput, TransferResult},
    },
    types::{
        address::{AccountAddress, AddressWithBalance},
        AccountBalance, OutputData, Transaction,
    },
    Account,
};
#[cfg(feature = "events")]
use crate::events::{
    types::{TransferProgressEvent, WalletEvent},
    EventEmitter,
};

use tokio::sync::{Mutex, RwLock};

use std::{ops::Deref, sync::Arc};

/// A thread guard over an account, so we can lock the account during operations.
#[derive(Debug, Clone)]
pub struct AccountHandle {
    account: Arc<RwLock<Account>>,
    // mutex to prevent multiple sync calls at the same or almost the same time, the u128 is a timestamp
    // if the last synced time was < `MIN_SYNC_INTERVAL` second ago, we don't sync, but only calculate the balance
    // again, because sending transactions can change that
    pub(crate) last_synced: Arc<Mutex<u128>>,
    #[cfg(feature = "events")]
    pub(crate) event_emitter: Arc<Mutex<EventEmitter>>,
}

impl AccountHandle {
    /// Create a new AccountHandle with an Account
    #[cfg(not(feature = "events"))]
    pub(crate) fn new(account: Account) -> Self {
        Self {
            account: Arc::new(RwLock::new(account)),
            last_synced: Default::default(),
        }
    }
    #[cfg(feature = "events")]
    pub(crate) fn new(account: Account, event_emitter: Arc<Mutex<EventEmitter>>) -> Self {
        Self {
            account: Arc::new(RwLock::new(account)),
            last_synced: Default::default(),
            event_emitter,
        }
    }

    /// Syncs the account by fetching new information from the nodes. Will also retry pending transactions and
    /// consolidate outputs if necessary.
    pub async fn sync(&self, options: Option<SyncOptions>) -> crate::Result<AccountBalance> {
        sync_account(self, &options.unwrap_or_default()).await
    }

    /// Consolidate outputs from addresses that have more outputs than the consolidation threshold
    async fn consolidate_outputs(&self) -> crate::Result<Vec<TransferResult>> {
        crate::account::operations::output_consolidation::consolidate_outputs(self).await
    }

    /// Send a transaction, if sending a message fails, the function will return None for the message_id, but the wallet
    /// will retry sending the transaction during syncing.
    /// ```ignore
    /// let outputs = vec![TransferOutput {
    ///     address: "atoi1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluehe53e".to_string(),
    ///     amount: 1_000_000,
    ///     output_kind: None,
    /// }];
    ///
    /// let res = account_handle
    ///     .send(
    ///         outputs,
    ///         Some(TransferOptions {
    ///             remainder_value_strategy: RemainderValueStrategy::ReuseAddress,
    ///             ..Default::default()
    ///         }),
    ///     )
    ///     .await?;
    /// println!("Transaction created: {}", res.1);
    /// if let Some(message_id) = res.0 {
    ///     println!("Message sent: {}", message_id);
    /// }
    /// ```
    pub async fn send(
        &self,
        outputs: Vec<TransferOutput>,
        options: Option<TransferOptions>,
    ) -> crate::Result<TransferResult> {
        // sync account before sending a transaction
        #[cfg(feature = "events")]
        {
            let account_index = self.account.read().await.index;
            self.event_emitter.lock().await.emit(
                account_index,
                WalletEvent::TransferProgress(TransferProgressEvent::SyncingAccount),
            );
        }
        if !options.clone().unwrap_or_default().skip_sync {
            sync_account(
                self,
                &SyncOptions {
                    automatic_output_consolidation: false,
                    ..Default::default()
                },
            )
            .await?;
        }
        send_transfer(self, outputs, options).await
    }

    // /// Reattaches or promotes a message to get it confirmed
    // pub async fn retry(message_id: MessageId, sync: bool) -> crate::Result<MessageId> {
    //     Ok(MessageId::from_str("")?)
    // }

    /// Generate addresses
    /// ```ignore
    /// let public_addresses = account_handle.generate_addresses(2, None).await?;
    /// // internal addresses are used for remainder outputs, if the RemainderValueStrategy for transfers is set to ChangeAddress
    /// let internal_addresses = account_handle
    ///     .generate_addresses(
    ///         1,
    ///         Some(AddressGenerationOptions {
    ///             internal: true,
    ///             ..Default::default()
    ///         }),
    ///     )
    ///     .await?;
    /// ```
    pub async fn generate_addresses(
        &self,
        amount: usize,
        options: Option<AddressGenerationOptions>,
    ) -> crate::Result<Vec<AccountAddress>> {
        let options = options.unwrap_or_default();
        address_generation::generate_addresses(self, amount, options).await
    }

    /// Returns all addresses of the account
    pub async fn list_addresses(&self) -> crate::Result<Vec<AccountAddress>> {
        let account = self.read().await;
        let mut all_addresses = account.public_addresses().clone();
        all_addresses.extend(account.internal_addresses().clone());
        Ok(all_addresses.to_vec())
    }

    /// Returns only addresses of the account with balance
    pub async fn list_addresses_with_balance(&self) -> crate::Result<Vec<AddressWithBalance>> {
        let account = self.read().await;
        Ok(account.addresses_with_balance().to_vec())
    }

    /// Returns all outputs of the account
    pub async fn list_outputs(&self) -> crate::Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all unspent outputs of the account
    pub async fn list_unspent_outputs(&self) -> crate::Result<Vec<OutputData>> {
        let account = self.read().await;
        let mut outputs = Vec::new();
        for output in account.unspent_outputs.values() {
            outputs.push(output.clone());
        }
        Ok(outputs)
    }

    /// Returns all transaction of the account
    pub async fn list_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction in account.transactions.values() {
            transactions.push(transaction.clone());
        }
        Ok(transactions)
    }

    /// Returns all pending transaction of the account
    pub async fn list_pending_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let account = self.read().await;
        let mut transactions = Vec::new();
        for transaction_id in &account.pending_transactions {
            if let Some(transaction) = account.transactions.get(transaction_id) {
                transactions.push(transaction.clone());
            }
        }
        Ok(transactions)
    }

    /// Get the total and available balance of an account
    pub async fn balance(&self) -> crate::Result<AccountBalance> {
        log::debug!("[BALANCE] get balance");
        let account = self.account.read().await;
        let total_balance: u64 = account.addresses_with_balance.iter().map(|a| a.balance()).sum();
        // for `available` get locked_outputs, sum outputs balance and subtract from total_balance
        log::debug!("[BALANCE] locked outputs: {:#?}", account.locked_outputs);
        let mut locked_balance = 0;
        let client = crate::client::get_client().await?;
        let network_id = client.get_network_id().await?;
        for locked_output in &account.locked_outputs {
            if let Some(output) = account.unspent_outputs.get(locked_output) {
                if output.network_id == network_id {
                    locked_balance += output.amount;
                }
            }
        }
        log::debug!(
            "[BALANCE] total_balance: {}, lockedbalance: {}",
            total_balance,
            locked_balance
        );
        Ok(AccountBalance {
            total: total_balance,
            available: total_balance - locked_balance,
        })
    }

    // Should only be called from the AccountManager so all accounts are on the same state
    pub(crate) async fn update_account_with_new_client(&self) -> crate::Result<()> {
        log::debug!("[UPDATE ACCOUNT WITH NEW CLIENT]");
        let client = crate::client::get_client().await?;
        let mut account = self.account.write().await;
        let bech32_hrp = client.get_bech32_hrp().await?;
        for address in &mut account.addresses_with_balance {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.public_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        for address in &mut account.internal_addresses {
            address.address.bech32_hrp = bech32_hrp.clone();
        }
        drop(account);
        // after we set the new client options we should sync the account because the network could have changed
        // we sync with all addresses, because otherwise the balance wouldn't get updated if an address doesn't has
        // balance also in the new network
        self.sync(Some(SyncOptions {
            sync_all_addresses: true,
            force_syncing: true,
            ..Default::default()
        }))
        .await?;
        Ok(())
    }

    /// Search addresses with funds
    /// `address_gap_limit` defines how many addresses without balance will be checked in each account, if an address
    /// has balance, the counter is reset
    /// Addresses that got crated during this operation and have a higher key_index than the latest one with balance,
    /// will be removed again, to keep the account size smaller
    pub(crate) async fn search_addresses_with_funds(&self, address_gap_limit: usize) -> crate::Result<AccountBalance> {
        search_addresses_with_funds(self, address_gap_limit).await
    }
}

// impl Deref so we can use `account_handle.read()` instead of `account_handle.account.read()`
impl Deref for AccountHandle {
    type Target = RwLock<Account>;
    fn deref(&self) -> &Self::Target {
        self.account.deref()
    }
}
