// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::*;
use chrono::prelude::{Local, TimeZone};
use iota::{Address as IotaAddress, MessageId as RustMessageId};
use iota_wallet::{
    address::AddressWrapper as RustAddressWrapper,
    message::{
        Message as RustWalletMessage, MessageType as RustMessageType,
        RemainderValueStrategy as RustRemainderValueStrategy, Transfer as RustTransfer,
    },
    signing::SignerType as RustSingerType,
};
use pyo3::prelude::*;
use std::{
    convert::{Into, TryInto},
    num::NonZeroU64,
    str::FromStr,
};

#[pymethods]
impl AccountSynchronizer {
    /// Number of address indexes that are generated.
    fn gap_limit(&mut self, limit: usize) {
        self.account_synchronizer = Some(self.account_synchronizer.take().unwrap().gap_limit(limit));
    }

    /// Skip saving new messages and addresses on the account object.
    /// The found data is returned on the `execute` call but won't be persisted on the database.
    fn skip_persistance(&mut self) {
        self.account_synchronizer = Some(self.account_synchronizer.take().unwrap().skip_persistance());
    }

    /// Initial address index to start syncing.
    fn address_index(&mut self, address_index: usize) {
        self.account_synchronizer = Some(self.account_synchronizer.take().unwrap().address_index(address_index));
    }

    /// Syncs account with the tangle.
    /// The account syncing process ensures that the latest metadata (balance, transactions)
    /// associated with an account is fetched from the tangle and is stored locally.
    fn execute(&mut self) -> Result<SyncedAccount> {
        let synced_account = crate::block_on(async { self.account_synchronizer.take().unwrap().execute().await })?;
        Ok(SyncedAccount { synced_account })
    }
}

#[pymethods]
impl Transfer {
    #[new]
    fn new(
        amount: u64,
        address: &str, // IotaAddress
        bench32_hrp: &str,
        indexation: Option<Indexation>,
        remainder_value_strategy: &str,
    ) -> Result<Self> {
        let address_wrapper = RustAddressWrapper::new(IotaAddress::try_from_bech32(address)?, bench32_hrp.to_string());
        let mut builder = RustTransfer::builder(address_wrapper, NonZeroU64::new(amount).unwrap());
        let strategy = match remainder_value_strategy {
            "ReuseAddress" => RustRemainderValueStrategy::ReuseAddress,
            "ChangeAddress" => RustRemainderValueStrategy::ChangeAddress,
            _ => RustRemainderValueStrategy::AccountAddress(RustAddressWrapper::new(
                IotaAddress::try_from_bech32(address)?,
                bench32_hrp.to_string(),
            )),
        };
        builder = builder.with_remainder_value_strategy(strategy);
        if let Some(indexation) = indexation {
            builder = builder.with_indexation(indexation.try_into()?);
        }
        Ok(Transfer {
            transfer: builder.finish(),
        })
    }
}

#[pymethods]
impl SyncedAccount {
    /// Send messages.
    fn transfer(&self, transfer_obj: Transfer) -> Result<WalletMessage> {
        crate::block_on(async { self.synced_account.transfer(transfer_obj.transfer).await?.try_into() })
    }

    /// Retry message.
    fn retry(&self, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.synced_account
                .retry(&RustMessageId::from_str(&message_id)?)
                .await?
                .try_into()
        })
    }

    /// Promote message.
    fn promote(&self, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.synced_account
                .promote(&RustMessageId::from_str(&message_id)?)
                .await?
                .try_into()
        })
    }

    /// Reattach message.
    fn reattach(&self, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.synced_account
                .reattach(&RustMessageId::from_str(&message_id)?)
                .await?
                .try_into()
        })
    }
}

#[pymethods]
impl AccountHandle {
    /// Returns the builder to setup the process to synchronize this account with the Tangle.
    fn sync(&self) -> AccountSynchronizer {
        let account_synchronizer = crate::block_on(async { self.account_handle.sync().await });
        AccountSynchronizer {
            account_synchronizer: Some(account_synchronizer),
        }
    }

    /// Gets a new unused address and links it to this account.
    fn generate_address(&self) -> Result<Address> {
        Ok(crate::block_on(async { self.account_handle.generate_address().await })?.into())
    }

    /// Synchronizes the account addresses with the Tangle and returns the latest address in the account,
    /// which is an address without balance.
    fn get_unused_address(&self) -> Result<Address> {
        Ok(crate::block_on(async { self.account_handle.get_unused_address().await })?.into())
    }

    /// Syncs the latest address with the Tangle and determines whether it's unused or not.
    /// An unused address is an address without balance and associated message history.
    /// Note that such address might have been used in the past, because the message history might have been pruned by
    /// the node.
    fn is_latest_address_unused(&self) -> Result<bool> {
        Ok(crate::block_on(async {
            self.account_handle.is_latest_address_unused().await
        })?)
    }

    /// Bridge to [Account#latest_address](struct.Account.html#method.latest_address).
    fn latest_address(&self) -> Address {
        crate::block_on(async { self.account_handle.latest_address().await }).into()
    }

    /// Bridge to [Account#addresses](struct.Account.html#method.addresses).
    fn addresses(&self) -> Vec<Address> {
        let addresses = crate::block_on(async { self.account_handle.addresses().await });
        addresses.into_iter().map(|address| address.into()).collect()
    }

    /// Bridge to [Account#balance](struct.Account.html#method.balance).
    fn balance(&self) -> AccountBalance {
        crate::block_on(async { self.account_handle.balance().await }).into()
    }

    /// Bridge to [Account#set_alias](struct.Account.html#method.set_alias).
    fn set_alias(&self, alias: &str) -> Result<()> {
        Ok(crate::block_on(async { self.account_handle.set_alias(alias).await })?)
    }

    /// Bridge to [Account#set_client_options](struct.Account.html#method.set_client_options).
    fn set_client_options(&self, options: ClientOptions) -> Result<()> {
        Ok(crate::block_on(async {
            self.account_handle.set_client_options(options.into()).await
        })?)
    }

    /// The number of messages associated with the account.
    fn message_count(&self, message_type: Option<&str>) -> usize {
        let message_type = match message_type {
            Some("Received") => Some(RustMessageType::Received),
            Some("Sent") => Some(RustMessageType::Sent),
            Some("Failed") => Some(RustMessageType::Failed),
            Some("Unconfirmed") => Some(RustMessageType::Unconfirmed),
            Some("Value") => Some(RustMessageType::Value),
            _ => None,
        };
        crate::block_on(async {
            self.account_handle
                .read()
                .await
                .list_messages(0, 0, message_type)
                .iter()
                .len()
        })
    }

    /// Bridge to [Account#list_messages](struct.Account.html#method.list_messages).
    /// This method clones the account's messages so when querying a large list of messages
    /// prefer using the `read` method to access the account instance.
    fn list_messages(
        &self,
        count: Option<usize>,
        from: Option<usize>,
        message_type: Option<&str>,
    ) -> Result<Vec<WalletMessage>> {
        let message_type = match message_type {
            Some("Received") => Some(RustMessageType::Received),
            Some("Sent") => Some(RustMessageType::Sent),
            Some("Failed") => Some(RustMessageType::Failed),
            Some("Unconfirmed") => Some(RustMessageType::Unconfirmed),
            Some("Value") => Some(RustMessageType::Value),
            _ => None,
        };
        let messages = crate::block_on(async {
            self.account_handle
                .list_messages(count.unwrap_or(0), from.unwrap_or(0), message_type)
                .await
        });

        let mut parsed_messages = Vec::new();
        for message in messages {
            parsed_messages.push(message.try_into()?);
        }

        Ok(parsed_messages)
    }

    /// Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    fn list_spent_addresses(&self) -> Vec<Address> {
        let addresses = crate::block_on(async { self.account_handle.list_spent_addresses().await });
        addresses.into_iter().map(|addr| addr.into()).collect()
    }

    /// Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    fn list_unspent_addresses(&self) -> Vec<Address> {
        let addresses = crate::block_on(async { self.account_handle.list_unspent_addresses().await });
        addresses.into_iter().map(|addr| addr.into()).collect()
    }

    /// Bridge to [Account#get_message](struct.Account.html#method.get_message).
    fn get_message(&self, message_id: &str) -> Result<Option<WalletMessage>> {
        let res: Result<Option<RustWalletMessage>> = crate::block_on(async {
            Ok(self
                .account_handle
                .get_message(&RustMessageId::from_str(&message_id)?)
                .await)
        });
        if let Some(message) = res? {
            Ok(Some(message.try_into()?))
        } else {
            Ok(None)
        }
    }
}

#[pymethods]
impl AccountInitialiser {
    /// Sets the account type.
    fn signer_type(&mut self, signer_type: &str) {
        let signer_type = match signer_type {
            "Stronghold" => RustSingerType::Stronghold,
            "LedgerNano" => RustSingerType::LedgerNano,
            "LedgerNanoSimulator" => RustSingerType::LedgerNanoSimulator,
            _ => RustSingerType::Custom(signer_type.to_string()),
        };
        self.account_initialiser = Some(self.account_initialiser.take().unwrap().signer_type(signer_type));
    }

    /// Defines the account alias. If not defined, we'll generate one.
    fn alias(&mut self, alias: &str) {
        self.account_initialiser = Some(self.account_initialiser.take().unwrap().alias(alias));
    }

    /// Time of account creation.
    fn created_at(&mut self, created_at: i64) {
        self.account_initialiser = Some(
            self.account_initialiser
                .take()
                .unwrap()
                .created_at(Local.timestamp(created_at, 0)),
        );
    }

    /// Messages associated with the seed.
    /// The account can be initialised with locally stored messages.
    fn messages(&mut self, messages: Vec<WalletMessage>) {
        self.account_initialiser = Some(
            self.account_initialiser.take().unwrap().messages(
                messages
                    .into_iter()
                    .map(|msg| {
                        // we use an empty bech32 HRP here because we update it later on wallet.rs
                        to_rust_message(msg, "".to_string(), &self.addresses)
                            .unwrap_or_else(|msg| panic!("AccountInitialiser: Message {:?} is invalid", msg))
                    })
                    .collect(),
            ),
        );
    }

    /// Address history associated with the seed.
    /// The account can be initialised with locally stored address history.
    fn addresses(&mut self, addresses: Vec<WalletAddress>) -> Result<()> {
        for address in &addresses {
            self.addresses.push(address.clone().try_into()?);
        }
        self.account_initialiser = Some(
            self.account_initialiser.take().unwrap().addresses(
                addresses
                    .into_iter()
                    .map(|address| {
                        address
                            .try_into()
                            .unwrap_or_else(|msg| panic!("AccountInitialiser: Address {:?} is invalid", msg))
                    })
                    .collect(),
            ),
        );
        Ok(())
    }

    /// Skips storing the account to the database.
    fn skip_persistance(&mut self) {
        self.account_initialiser = Some(self.account_initialiser.take().unwrap().skip_persistance());
    }

    /// Initialises the account.
    fn initialise(&mut self) -> Result<AccountHandle> {
        let account_handle = crate::block_on(async { self.account_initialiser.take().unwrap().initialise().await })?;
        Ok(AccountHandle { account_handle })
    }
}
