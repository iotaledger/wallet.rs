// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::*;
use chrono::prelude::{Local, TimeZone};
use iota_client::bee_message::MessageId as RustMessageId;
use iota_wallet::{
    address::{parse as parse_address, OutputKind as RustOutputKind},
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
    vec,
};

#[pymethods]
impl AccountSynchronizer {
    /// Number of address indexes that are generated.
    fn gap_limit(&mut self, limit: usize) {
        self.account_synchronizer = Some(self.account_synchronizer.take().unwrap().gap_limit(limit));
    }

    /// Skip saving new messages and addresses on the account object.
    /// The found data is returned on the `execute` call but won't be persisted on the database.
    fn skip_persistence(&mut self) {
        self.account_synchronizer = Some(self.account_synchronizer.take().unwrap().skip_persistence());
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
impl SyncedAccount {
    /// Get the `AccountHandle` of this account
    fn account_handle(&self) -> AccountHandle {
        AccountHandle {
            account_handle: self.synced_account.account_handle().clone(),
        }
    }

    /// Get the deposit_address of this account
    fn deposit_address(&self) -> Address {
        self.synced_account.deposit_address().clone().into()
    }

    /// Get the messages of this account
    fn messages(&self) -> Result<Vec<WalletMessage>> {
        let mut parsed_messages = Vec::new();
        for message in self.synced_account.messages().clone() {
            parsed_messages.push(message.try_into()?);
        }

        Ok(parsed_messages)
    }

    /// Get the addresses of this account
    fn addresses(&self) -> Vec<Address> {
        let addresses = self.synced_account.addresses().clone();
        addresses.into_iter().map(|address| address.into()).collect()
    }
}

#[pymethods]
impl Transfer {
    #[new]
    fn new(
        amount: u64,
        address: &str, // IotaAddress
        indexation: Option<Indexation>,
        remainder_value_strategy: Option<&str>,
        skip_sync: Option<bool>,
        output_kind: Option<&str>,
    ) -> Result<Self> {
        let address_wrapper = parse_address(address)?;
        let output_kind = match output_kind {
            Some("SignatureLockedSingle") => Some(RustOutputKind::SignatureLockedSingle),
            Some("SignatureLockedDustAllowance") => Some(RustOutputKind::SignatureLockedDustAllowance),
            _ => None,
        };
        let mut builder = RustTransfer::builder(address_wrapper, NonZeroU64::new(amount).unwrap(), output_kind);
        let strategy = match remainder_value_strategy {
            Some("ReuseAddress") => RustRemainderValueStrategy::ReuseAddress,
            Some("ChangeAddress") => RustRemainderValueStrategy::ChangeAddress,
            _ => RustRemainderValueStrategy::ChangeAddress,
        };
        builder = builder.with_remainder_value_strategy(strategy);
        if let Some(indexation) = indexation {
            builder = builder.with_indexation(indexation.try_into()?);
        }
        if skip_sync.unwrap_or_default() {
            builder = builder.with_skip_sync();
        }
        Ok(Transfer {
            transfer: builder.finish(),
        })
    }
}
#[pymethods]
impl TransferWithOutputs {
    #[new]
    fn new(
        outputs: Vec<TransferOutput>,
        indexation: Option<Indexation>,
        remainder_value_strategy: Option<&str>,
        skip_sync: Option<bool>,
    ) -> Result<Self> {
        let mut rust_outputs = Vec::new();
        for output in outputs {
            rust_outputs.push(output.try_into()?);
        }

        let mut builder = RustTransfer::builder_with_outputs(rust_outputs)?;
        let strategy = match remainder_value_strategy {
            Some("ReuseAddress") => RustRemainderValueStrategy::ReuseAddress,
            Some("ChangeAddress") => RustRemainderValueStrategy::ChangeAddress,
            _ => RustRemainderValueStrategy::ChangeAddress,
        };
        builder = builder.with_remainder_value_strategy(strategy);
        if let Some(indexation) = indexation {
            builder = builder.with_indexation(indexation.try_into()?);
        }
        if skip_sync.unwrap_or_default() {
            builder = builder.with_skip_sync();
        }
        Ok(TransferWithOutputs {
            transfer: builder.finish(),
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

    /// Send messages.
    fn transfer(&self, transfer_obj: Transfer) -> Result<WalletMessage> {
        crate::block_on(async { self.account_handle.transfer(transfer_obj.transfer).await?.try_into() })
    }

    /// Send messages.
    fn transfer_with_outputs(&self, transfer_obj: TransferWithOutputs) -> Result<WalletMessage> {
        crate::block_on(async { self.account_handle.transfer(transfer_obj.transfer).await?.try_into() })
    }

    /// Retry message.
    fn retry(&self, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_handle
                .retry(&RustMessageId::from_str(message_id)?)
                .await?
                .try_into()
        })
    }

    /// Promote message.
    fn promote(&self, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_handle
                .promote(&RustMessageId::from_str(message_id)?)
                .await?
                .try_into()
        })
    }

    /// Reattach message.
    fn reattach(&self, message_id: &str) -> Result<WalletMessage> {
        crate::block_on(async {
            self.account_handle
                .reattach(&RustMessageId::from_str(message_id)?)
                .await?
                .try_into()
        })
    }
}

#[pymethods]
impl AccountHandle {
    /// Bridge to [Account#id](struct.Account.html#method.id).
    /// Returns the account ID.
    fn id(&self) -> String {
        crate::block_on(async { self.account_handle.id().await })
    }

    /// Bridge to [Account#signer_type](struct.Account.html#method.signer_type).
    /// Return the singer type of this account.
    fn signer_type(&self) -> String {
        match crate::block_on(async { self.account_handle.signer_type().await }) {
            RustSingerType::Stronghold => "Stronghold".to_string(),
            RustSingerType::LedgerNano => "LedgerNano".to_string(),
            RustSingerType::LedgerNanoSimulator => "LedgerNanoSimulator".to_string(),
            RustSingerType::Custom(s) => s,
        }
    }

    /// Bridge to [Account#index](struct.Account.html#method.index).
    /// Return the account index.
    fn index(&self) -> usize {
        crate::block_on(async { self.account_handle.index().await })
    }

    /// Bridge to [Account#alias](struct.Account.html#method.alias).
    /// Return the account alias.
    fn alias(&self) -> String {
        crate::block_on(async { self.account_handle.alias().await })
    }

    /// Bridge to [Account#created_at](struct.Account.html#method.created_at).
    /// Return the created UNIX timestamp
    fn created_at(&self) -> i64 {
        crate::block_on(async { self.account_handle.created_at().await }).timestamp()
    }

    /// Bridge to [Account#last_synced_at](struct.Account.html#method.last_synced_at).
    /// Return the last synced UNIX timestamp
    fn last_synced_at(&self) -> Option<i64> {
        crate::block_on(async { self.account_handle.last_synced_at().await }).map(|t| t.timestamp())
    }

    /// Bridge to [Account#client_options](struct.Account.html#method.client_options).
    /// Return the client options of this account
    fn client_options(&self) -> ClientOptions {
        crate::block_on(async { self.account_handle.client_options().await }).into()
    }

    // #[doc = "Bridge to [Account#bech32_hrp](struct.Account.html#method.bech32_hrp)."] => bech32_hrp => String
    fn bech32_hrp(&self) -> String {
        crate::block_on(async { self.account_handle.bech32_hrp().await })
    }

    /// Consolidate outputs.
    fn consolidate_outputs(&self, include_dust_allowance_outputs: bool) -> Result<Vec<WalletMessage>> {
        let rust_messages = crate::block_on(async {
            self.account_handle
                .consolidate_outputs(include_dust_allowance_outputs)
                .await
        })?;
        let mut messages = Vec::new();
        for message in rust_messages {
            messages.push(message.try_into()?);
        }
        Ok(messages)
    }

    /// Gets a new unused address and links it to this account.
    fn generate_address(&self) -> Result<Address> {
        Ok(crate::block_on(async { self.account_handle.generate_address().await })?.into())
    }

    /// Gets a new unused address and links it to this account.
    fn generate_addresses(&self, amount: usize) -> Result<Vec<Address>> {
        Ok(
            crate::block_on(async { self.account_handle.generate_addresses(amount).await })?
                .into_iter()
                .map(|a| a.into())
                .collect(),
        )
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
    fn balance(&self) -> Result<AccountBalance> {
        Ok(crate::block_on(async { self.account_handle.balance().await })?.into())
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
    fn message_count(&self, message_type: Option<&str>) -> Result<usize> {
        let message_type = match message_type {
            Some("Received") => Some(RustMessageType::Received),
            Some("Sent") => Some(RustMessageType::Sent),
            Some("Failed") => Some(RustMessageType::Failed),
            Some("Unconfirmed") => Some(RustMessageType::Unconfirmed),
            Some("Value") => Some(RustMessageType::Value),
            Some("Confirmed") => Some(RustMessageType::Confirmed),
            _ => None,
        };
        crate::block_on(async {
            Ok(self
                .account_handle
                .read()
                .await
                .list_messages(0, 0, message_type)
                .await?
                .iter()
                .len())
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
            Some("Confirmed") => Some(RustMessageType::Confirmed),
            _ => None,
        };
        let messages = crate::block_on(async {
            self.account_handle
                .list_messages(count.unwrap_or(0), from.unwrap_or(0), message_type)
                .await
        });

        let mut parsed_messages = Vec::new();
        for message in messages? {
            parsed_messages.push(message.try_into()?);
        }

        Ok(parsed_messages)
    }

    /// Bridge to [Account#list_spent_addresses](struct.Account.html#method.list_spent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    fn list_spent_addresses(&self) -> Result<Vec<Address>> {
        let addresses = crate::block_on(async { self.account_handle.list_spent_addresses().await });
        Ok(addresses?.into_iter().map(|addr| addr.into()).collect())
    }

    /// Bridge to [Account#list_unspent_addresses](struct.Account.html#method.list_unspent_addresses).
    /// This method clones the account's addresses so when querying a large list of addresses
    /// prefer using the `read` method to access the account instance.
    fn list_unspent_addresses(&self) -> Result<Vec<Address>> {
        let addresses = crate::block_on(async { self.account_handle.list_unspent_addresses().await });
        Ok(addresses?.into_iter().map(|addr| addr.into()).collect())
    }

    /// Bridge to [Account#get_message](struct.Account.html#method.get_message).
    fn get_message(&self, message_id: &str) -> Result<Option<WalletMessage>> {
        let res: Result<Option<RustWalletMessage>> = crate::block_on(async {
            Ok(self
                .account_handle
                .get_message(&RustMessageId::from_str(message_id)?)
                .await)
        });
        if let Some(message) = res? {
            Ok(Some(message.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// Bridge to [Account#get_node_info](struct.Account.html#method.get_node_info).
    fn get_node_info(&self, url: Option<&str>, auth_options: Option<Vec<&str>>) -> Result<NodeInfoWrapper> {
        let mut jwt = None;
        let mut auth = None;
        if let Some(opt) = auth_options {
            if opt.len() == 1 {
                jwt = Some(opt[0]);
            } else if opt.len() == 2 {
                auth = Some((opt[0], opt[1]));
            } else if opt.len() == 3 {
                jwt = Some(opt[0]);
                auth = Some((opt[1], opt[2]));
            }
        }

        Ok(crate::block_on(async { self.account_handle.get_node_info(url, jwt, auth).await })?.into())
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
        let mut account_initialiser = self.account_initialiser.take().unwrap();
        let messages = messages
            .into_iter()
            .map(|msg| {
                // we use an empty bech32 HRP here because we update it later on wallet.rs
                crate::block_on(to_rust_message(
                    msg,
                    "".to_string(),
                    self.accounts.clone(),
                    "",
                    &self.addresses,
                    &account_initialiser.client_options,
                ))
                .unwrap_or_else(|msg| panic!("AccountInitialiser: Message {msg:?} is invalid"))
            })
            .collect();
        account_initialiser = account_initialiser.messages(messages);
        self.account_initialiser.replace(account_initialiser);
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
                            .unwrap_or_else(|msg| panic!("AccountInitialiser: Address {msg:?} is invalid"))
                    })
                    .collect(),
            ),
        );
        Ok(())
    }

    /// Skips storing the account to the database.
    fn skip_persistence(&mut self) {
        self.account_initialiser = Some(self.account_initialiser.take().unwrap().skip_persistence());
    }

    /// Initialises the account.
    fn initialise(&mut self) -> Result<AccountHandle> {
        let account_handle = crate::block_on(async { self.account_initialiser.take().unwrap().initialise().await })?;
        Ok(AccountHandle { account_handle })
    }
}
