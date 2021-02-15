public class AccountHandle{
    inner: Arc<RwLock<Account>>,
    locked_addresses: Arc<Mutex<Vec<AddressWrapper>>>,
}

/// Returns the builder to setup the process to synchronize this account with the Tangle.
    pub async fn sync(&self) -> AccountSynchronizer {
        AccountSynchronizer::new(self.clone()).await
    }

    /// Gets a new unused address and links it to this account.
    pub async fn generate_address(&self) -> crate::Result<Address> {
        let mut account = self.inner.write().await;
        self.generate_address_internal(&mut account).await
    }

    /// Generates an address without locking the account.
    pub(crate) async fn generate_address_internal(
        &self,
        account: &mut RwLockWriteGuard<'_, Account>,
    ) -> crate::Result<Address> {
        let address = crate::address::get_new_address(&account, GenerateAddressMetadata { syncing: false }).await?;

        account
            .do_mut(|account| {
                account.addresses.push(address.clone());
                Ok(())
            })
            .await?;

        let _ = crate::monitor::monitor_address_balance(self.clone(), address.address());

        Ok(address)
    }

    /// Synchronizes the account addresses with the Tangle and returns the latest address in the account,
    /// which is an address without balance.
    pub async fn get_unused_address(&self) -> crate::Result<Address> {
        self.sync()
            .await
            .steps(vec![AccountSynchronizeStep::SyncAddresses])
            .execute()
            .await?;
        // safe to clone since the `sync` guarantees a latest unused address
        Ok(self.latest_address().await)
    }

    /// Syncs the latest address with the Tangle and determines whether it's unused or not.
    /// An unused address is an address without balance and associated message history.
    /// Note that such address might have been used in the past, because the message history might have been pruned by
    /// the node.
    pub async fn is_latest_address_unused(&self) -> crate::Result<bool> {
        let mut latest_address = self.latest_address().await;
        let bech32_hrp = latest_address.address().bech32_hrp().to_string();
        sync::sync_address(&*self.inner.read().await, &mut latest_address, bech32_hrp).await?;
        Ok(*latest_address.balance() == 0 && latest_address.outputs().is_empty())
    }

    /// Bridge to [Account#latest_address](struct.Account.html#method.latest_address).
    pub async fn latest_address(&self) -> Address {
        self.inner.read().await.latest_address().clone()
    }

    /// Bridge to [Account#balance](struct.Account.html#method.balance).
    pub async fn balance(&self) -> AccountBalance {
        self.inner.read().await.balance()
    }

    /// Bridge to [Account#set_alias](struct.Account.html#method.set_alias).
    pub async fn set_alias(&self, alias: impl AsRef<str>) -> crate::Result<()> {
        self.inner.write().await.set_alias(alias).await
    }

    /// Bridge to [Account#set_client_options](struct.Account.html#method.set_client_options).
    pub async fn set_client_options(&self, options: ClientOptions) -> crate::Result<()> {
        self.inner.write().await.set_client_options(options).await
    }

    /// Bridge to [Account#list_messages](struct.Account.html#method.list_messages).
    /// This method clones the account's messages so when querying a large list of messages
    /// prefer using the `read` method to access the account instance.
    pub async fn list_messages(&self, count: usize, from: usize, message_type: Option<MessageType>) -> Vec<Message> {
        self.inner
            .read()
            .await
            .list_messages(count, from, message_type)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    pub async fn list_spent_addresses(&self) -> Vec<Address> {
        self.inner
            .read()
            .await
            .list_spent_addresses()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    pub async fn list_unspent_addresses(&self) -> Vec<Address> {
        self.inner
            .read()
            .await
            .list_unspent_addresses()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Bridge to [Account#get_message](struct.Account.html#method.get_message).
    pub async fn get_message(&self, message_id: &MessageId) -> Option<Message> {
        self.inner.read().await.get_message(message_id).cloned()
    }