// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::{
    account::AccountIdentifier,
    account_manager::{AccountManager, MigrationDataFinder},
    iota_migration::{client::migration::add_tryte_checksum, transaction::bundled::Address as LegacyAddress},
    message::{Message as WalletMessage, Transfer},
    Result,
};
use futures::{Future, FutureExt};
use iota_client::bee_message::MessageId;
use iota_migration::{
    client::extended::AddressInput,
    ternary::TryteBuf,
    transaction::bundled::{Address as TryteAddress, BundledTransactionField},
};
use zeroize::Zeroize;

use std::{
    any::Any,
    convert::TryInto,
    num::NonZeroU64,
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
    time::Duration,
};

mod message;
pub use message::*;

/// The Wallet message handler.
pub struct WalletMessageHandler {
    account_manager: AccountManager,
}

fn panic_to_response_message(panic: Box<dyn Any>) -> ResponseType {
    let msg = if let Some(message) = panic.downcast_ref::<String>() {
        format!("Internal error: {}", message)
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        format!("Internal error: {}", message)
    } else {
        "Internal error".to_string()
    };
    let current_backtrace = backtrace::Backtrace::new();
    ResponseType::Panic(format!("{}\n\n{:?}", msg, current_backtrace))
}

fn convert_panics<F: FnOnce() -> Result<ResponseType>>(f: F) -> Result<ResponseType> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic) => Ok(panic_to_response_message(panic)),
    }
}

async fn convert_async_panics<F>(f: impl FnOnce() -> F) -> Result<ResponseType>
where
    F: Future<Output = Result<ResponseType>>,
{
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => result,
        Err(panic) => Ok(panic_to_response_message(panic)),
    }
}

impl WalletMessageHandler {
    /// Creates a new instance of the message handler with the default account manager.
    pub async fn new() -> Result<Self> {
        let instance = Self {
            account_manager: AccountManager::builder().finish().await?,
        };
        Ok(instance)
    }

    /// Creates a new instance of the message handler with the specified account manager.
    pub fn with_manager(account_manager: AccountManager) -> Self {
        Self { account_manager }
    }

    /// Handles a message.
    pub async fn handle(&self, mut message: Message) {
        let response: Result<ResponseType> = match message.message_type_mut() {
            MessageType::RemoveAccount(account_id) => {
                convert_async_panics(|| async { self.remove_account(account_id).await }).await
            }
            MessageType::CreateAccount(account) => {
                convert_async_panics(|| async { self.create_account(account).await }).await
            }
            MessageType::GetAccount(account_id) => {
                convert_async_panics(|| async { self.get_account(account_id).await }).await
            }
            MessageType::GetAccounts => convert_async_panics(|| async { self.get_accounts().await }).await,
            MessageType::CallAccountMethod { account_id, method } => {
                convert_async_panics(|| async { self.call_account_method(account_id, method).await }).await
            }
            MessageType::SyncAccounts {
                address_index,
                gap_limit,
                account_discovery_threshold,
            } => {
                convert_async_panics(|| async {
                    self.sync_accounts(address_index, gap_limit, account_discovery_threshold)
                        .await
                })
                .await
            }
            MessageType::Reattach { account_id, message_id } => {
                convert_async_panics(|| async { self.reattach(account_id, message_id).await }).await
            }
            MessageType::Backup { destination, password } => {
                convert_async_panics(|| async {
                    let res = self.backup(destination, password.to_string()).await;
                    password.zeroize();
                    res
                })
                .await
            }
            MessageType::RestoreBackup { backup_path, password } => {
                let res =
                    convert_async_panics(|| async { self.restore_backup(backup_path, password.to_string()).await })
                        .await;
                password.zeroize();
                res
            }
            MessageType::SetStoragePassword(password) => {
                convert_async_panics(|| async { self.set_storage_password(password).await }).await
            }
            #[cfg(feature = "stronghold")]
            MessageType::SetStrongholdPassword(password) => {
                convert_async_panics(|| async { self.set_stronghold_password(password).await }).await
            }
            #[cfg(feature = "stronghold")]
            MessageType::SetStrongholdPasswordClearInterval(interval) => {
                convert_async_panics(|| async {
                    crate::set_stronghold_password_clear_interval(*interval).await;
                    Ok(ResponseType::StrongholdPasswordClearIntervalSet)
                })
                .await
            }
            #[cfg(feature = "stronghold")]
            MessageType::GetStrongholdStatus => {
                convert_async_panics(|| async {
                    let status =
                        crate::get_stronghold_status(&self.account_manager.stronghold_snapshot_path().await?).await;
                    Ok(ResponseType::StrongholdStatus(status))
                })
                .await
            }
            #[cfg(feature = "stronghold")]
            MessageType::LockStronghold => {
                convert_async_panics(|| async {
                    crate::lock_stronghold(&self.account_manager.stronghold_snapshot_path().await?, true).await?;
                    Ok(ResponseType::LockedStronghold)
                })
                .await
            }
            MessageType::GenerateMnemonic => convert_panics(|| {
                self.account_manager
                    .generate_mnemonic()
                    .map(ResponseType::GeneratedMnemonic)
            }),
            MessageType::VerifyMnemonic(mnemonic) => convert_panics(|| {
                self.account_manager
                    .verify_mnemonic(mnemonic)
                    .map(|_| ResponseType::VerifiedMnemonic)
            }),
            MessageType::StoreMnemonic { signer_type, mnemonic } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .store_mnemonic(signer_type.clone(), mnemonic.clone())
                        .await
                        .map(|_| ResponseType::StoredMnemonic)
                })
                .await
            }
            MessageType::IsLatestAddressUnused => {
                convert_async_panics(|| async {
                    self.account_manager
                        .is_latest_address_unused()
                        .await
                        .map(ResponseType::AreAllLatestAddressesUnused)
                })
                .await
            }
            #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
            MessageType::GetLedgerStatus(is_simulator) => {
                convert_async_panics(|| async {
                    Ok(ResponseType::LedgerStatus(
                        crate::get_ledger_status(*is_simulator).await,
                    ))
                })
                .await
            }
            MessageType::DeleteStorage => {
                convert_async_panics(|| async move {
                    self.account_manager.delete_internal().await?;
                    Ok(ResponseType::DeletedStorage)
                })
                .await
            }
            MessageType::SendTransfer { account_id, transfer } => {
                convert_async_panics(|| async { self.send_transfer(account_id, transfer.clone().finish()).await }).await
            }
            MessageType::InternalTransfer {
                from_account_id,
                to_account_id,
                amount,
            } => {
                convert_async_panics(|| async { self.internal_transfer(from_account_id, to_account_id, *amount).await })
                    .await
            }
            #[cfg(feature = "stronghold")]
            MessageType::ChangeStrongholdPassword {
                current_password,
                new_password,
            } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .change_stronghold_password(current_password.to_string(), new_password.to_string())
                        .await?;
                    current_password.zeroize();
                    new_password.zeroize();
                    Ok(ResponseType::StrongholdPasswordChanged)
                })
                .await
            }
            MessageType::SetClientOptions(options) => {
                convert_async_panics(|| async {
                    self.account_manager.set_client_options(*options.clone()).await?;
                    Ok(ResponseType::UpdatedAllClientOptions)
                })
                .await
            }
            MessageType::GetMigrationData {
                nodes,
                permanode,
                seed,
                security_level,
                gap_limit,
                initial_address_index,
            } => {
                convert_async_panics(|| async {
                    let nodes = nodes.iter().map(String::as_ref).collect::<Vec<&str>>();
                    let mut finder = MigrationDataFinder::new(&nodes, seed)?;
                    if let Some(level) = security_level {
                        finder = finder.with_security_level(*level);
                    }
                    if let Some(permanode) = permanode {
                        finder = finder.with_permanode(permanode);
                    }
                    if let Some(initial_address_index) = initial_address_index {
                        finder = finder.with_initial_address_index(*initial_address_index);
                    }
                    if let Some(gap_limit) = gap_limit {
                        finder = finder.with_gap_limit(*gap_limit);
                    }
                    let data = self.account_manager.get_migration_data(finder).await?;
                    seed.zeroize();
                    Ok(ResponseType::MigrationData(data.into()))
                })
                .await
            }
            MessageType::GetLedgerMigrationData {
                nodes,
                permanode,
                addresses,
                security_level,
            } => {
                convert_async_panics(|| async {
                    use serde::Deserialize;
                    #[derive(Deserialize)]
                    struct AddressIndex {
                        pub address: String,
                        pub index: u64,
                    }
                    let address_inputs = addresses
                        .iter()
                        .map(|address| {
                            let address_index: AddressIndex = serde_json::from_str(address)?;
                            Ok(AddressInput {
                                address: TryteAddress::from_inner_unchecked(
                                    TryteBuf::try_from_str(&address_index.address)
                                        .map_err(|_| crate::error::Error::TernaryError)?
                                        .as_trits()
                                        .encode(),
                                ),
                                index: address_index.index,
                                security_lvl: security_level.unwrap_or(2),
                            })
                        })
                        .collect::<Result<Vec<AddressInput>>>();

                    let nodes = nodes.iter().map(String::as_ref).collect::<Vec<&str>>();

                    let data = self
                        .account_manager
                        .get_ledger_migration_data(address_inputs?, nodes, permanode.clone())
                        .await?;
                    Ok(ResponseType::MigrationData(data.into()))
                })
                .await
            }
            MessageType::CreateMigrationBundle {
                seed,
                input_address_indexes,
                mine,
                timeout_secs,
                offset,
                log_file_name,
            } => {
                convert_async_panics(|| async {
                    let bundle = self
                        .account_manager
                        .create_migration_bundle(
                            seed,
                            input_address_indexes,
                            *mine,
                            Duration::from_secs(*timeout_secs),
                            *offset,
                            log_file_name,
                        )
                        .await?;
                    seed.zeroize();
                    Ok(ResponseType::CreatedMigrationBundle(bundle))
                })
                .await
            }
            MessageType::SendMigrationBundle {
                nodes,
                bundle_hash,
                mwm,
            } => {
                convert_async_panics(|| async {
                    let nodes = nodes.iter().map(String::as_ref).collect::<Vec<&str>>();
                    let migrated_bundle = self
                        .account_manager
                        .send_migration_bundle(&nodes, bundle_hash, *mwm)
                        .await?;
                    Ok(ResponseType::SentMigrationBundle(migrated_bundle))
                })
                .await
            }
            MessageType::SendLedgerMigrationBundle { nodes, bundle, mwm } => {
                convert_async_panics(|| async {
                    let nodes = nodes.iter().map(String::as_ref).collect::<Vec<&str>>();
                    let migrated_bundle = self
                        .account_manager
                        .send_ledger_migration_bundle(&nodes, bundle.clone(), *mwm)
                        .await?;
                    Ok(ResponseType::SentMigrationBundle(migrated_bundle))
                })
                .await
            }
            MessageType::GetSeedChecksum(seed) => convert_panics(|| {
                let checksum = AccountManager::get_seed_checksum(seed.clone())?;
                Ok(ResponseType::SeedChecksum(checksum))
            }),
            MessageType::GetMigrationAddress {
                ledger_prompt,
                account_id,
            } => {
                convert_async_panics(|| async {
                    let address = self
                        .account_manager
                        .get_migration_address(*ledger_prompt, account_id.clone())
                        .await?;
                    Ok(ResponseType::MigrationAddress(address))
                })
                .await
            }
            MessageType::MineBundle {
                prepared_bundle,
                spent_bundle_hashes,
                security_level,
                timeout,
                offset,
            } => {
                convert_async_panics(|| async {
                    let mined_bundle = self
                        .account_manager
                        .mine_bundle(
                            prepared_bundle.to_vec(),
                            spent_bundle_hashes.to_vec(),
                            *security_level,
                            *timeout,
                            *offset,
                        )
                        .await?;
                    Ok(ResponseType::MineBundle(mined_bundle))
                })
                .await
            }
            MessageType::GetLegacyAddressChecksum(address) => convert_panics(|| {
                let address = LegacyAddress::from_inner_unchecked(
                    TryteBuf::try_from_str(address)
                        .map_err(|_| crate::Error::InvalidAddress)?
                        .as_trits()
                        .encode(),
                );
                let address_with_checksum = add_tryte_checksum(address)?;
                Ok(ResponseType::GetLegacyAddressChecksum(address_with_checksum))
            }),
            MessageType::StartBackgroundSync {
                polling_interval,
                automatic_output_consolidation,
            } => {
                convert_async_panics(|| async {
                    self.account_manager
                        .start_background_sync(*polling_interval, *automatic_output_consolidation)
                        .await?;
                    Ok(ResponseType::Ok(()))
                })
                .await
            }
            MessageType::StopBackgroundSync => {
                convert_async_panics(|| async {
                    self.account_manager.stop_background_sync()?;
                    Ok(ResponseType::Ok(()))
                })
                .await
            }
            #[cfg(feature = "participation")]
            MessageType::Participate {
                account_identifier,
                participations,
            } => {
                convert_async_panics(|| async {
                    let messages = self
                        .account_manager
                        .participate(account_identifier.clone(), participations.clone())
                        .await?;
                    Ok(ResponseType::SentParticipation(messages))
                })
                .await
            }
            #[cfg(feature = "participation")]
            MessageType::StopParticipating {
                account_identifier,
                event_ids,
            } => {
                convert_async_panics(|| async {
                    let messages = self
                        .account_manager
                        .stop_participating(account_identifier.clone(), event_ids.clone())
                        .await?;
                    Ok(ResponseType::SentParticipation(messages))
                })
                .await
            }
            #[cfg(feature = "participation")]
            MessageType::ParticipateWithRemainingFunds {
                account_identifier,
                participations,
            } => {
                convert_async_panics(|| async {
                    let messages = self
                        .account_manager
                        .participate_with_remaining_funds(account_identifier.clone(), participations.clone())
                        .await?;
                    Ok(ResponseType::SentParticipation(messages))
                })
                .await
            }
            #[cfg(feature = "participation")]
            MessageType::GetParticipationOverview => {
                convert_async_panics(|| async {
                    let overview = self.account_manager.get_participation_overview().await?;
                    Ok(ResponseType::ParticipationOverview(overview))
                })
                .await
            }
            #[cfg(feature = "participation")]
            MessageType::GetParticipationEvents => {
                convert_async_panics(|| async {
                    let events_data = self.account_manager.get_participation_events().await?;
                    Ok(ResponseType::EventsData(events_data))
                })
                .await
            }
        };

        let response = match response {
            Ok(r) => r,
            Err(e) => ResponseType::Error(e),
        };
        let _ = message
            .response_tx
            .send(Response::new(message.id().to_string(), message.message_type, response));
    }

    async fn backup(&self, destination_path: &Path, password: String) -> Result<ResponseType> {
        self.account_manager.backup(destination_path, password).await?;
        Ok(ResponseType::BackupSuccessful)
    }

    async fn restore_backup(&self, backup_path: &str, password: String) -> Result<ResponseType> {
        self.account_manager.import_accounts(backup_path, password).await?;
        Ok(ResponseType::BackupRestored)
    }

    async fn reattach(&self, account_id: &AccountIdentifier, message_id: &str) -> Result<ResponseType> {
        let parsed_message_id = MessageId::new(
            message_id.as_bytes()[..]
                .try_into()
                .map_err(|_| crate::Error::InvalidMessageId)?,
        );
        self.account_manager
            .reattach(account_id.clone(), &parsed_message_id)
            .await?;
        Ok(ResponseType::Reattached(message_id.to_string()))
    }

    async fn sync_accounts(
        &self,
        address_index: &Option<usize>,
        gap_limit: &Option<usize>,
        account_discovery_threshold: &Option<usize>,
    ) -> Result<ResponseType> {
        let mut synchronizer = self.account_manager.sync_accounts()?;
        if let Some(address_index) = address_index {
            synchronizer = synchronizer.address_index(*address_index);
        }
        if let Some(gap_limit) = gap_limit {
            synchronizer = synchronizer.gap_limit(*gap_limit);
        }
        if let Some(account_discovery_threshold) = account_discovery_threshold {
            synchronizer = synchronizer.account_discovery_threshold(*account_discovery_threshold);
        }
        let synced = synchronizer.execute().await?;
        Ok(ResponseType::SyncedAccounts(synced))
    }

    async fn call_account_method(
        &self,
        account_id: &AccountIdentifier,
        method: &AccountMethod,
    ) -> Result<ResponseType> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;

        match method {
            AccountMethod::GenerateAddress => {
                let address = account_handle.generate_address().await?;
                Ok(ResponseType::GeneratedAddress(address))
            }
            AccountMethod::GetUnusedAddress => {
                let address = account_handle.get_unused_address().await?;
                Ok(ResponseType::UnusedAddress(address))
            }
            AccountMethod::ListMessages {
                count,
                from,
                message_type,
            } => {
                let messages: Vec<WalletMessage> = account_handle
                    .read()
                    .await
                    .list_messages(*count, *from, message_type.clone())
                    .await?;
                Ok(ResponseType::Messages(messages))
            }
            AccountMethod::ListAddresses => {
                let addresses = account_handle.addresses().await;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::ListSpentAddresses => {
                let addresses = account_handle.list_spent_addresses().await?;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::ListUnspentAddresses => {
                let addresses = account_handle.list_unspent_addresses().await?;
                Ok(ResponseType::Addresses(addresses))
            }
            AccountMethod::GetBalance => Ok(ResponseType::Balance(account_handle.read().await.balance().await?)),
            AccountMethod::GetLatestAddress => Ok(ResponseType::LatestAddress(
                account_handle.read().await.latest_address().clone(),
            )),
            AccountMethod::SyncAccount {
                address_index,
                gap_limit,
                skip_persistence,
            } => {
                let mut synchronizer = account_handle.sync().await;
                if let Some(address_index) = address_index {
                    synchronizer = synchronizer.address_index(*address_index);
                }
                if let Some(gap_limit) = gap_limit {
                    synchronizer = synchronizer.gap_limit(*gap_limit);
                }
                if let Some(skip_persistence) = skip_persistence {
                    if *skip_persistence {
                        synchronizer = synchronizer.skip_persistence();
                    }
                }
                let synced = synchronizer.execute().await?;
                Ok(ResponseType::SyncedAccount(synced))
            }
            AccountMethod::IsLatestAddressUnused => Ok(ResponseType::IsLatestAddressUnused(
                account_handle.is_latest_address_unused().await?,
            )),
            AccountMethod::SetAlias(alias) => {
                account_handle.set_alias(alias).await?;
                Ok(ResponseType::UpdatedAlias)
            }
            AccountMethod::SetClientOptions(options) => {
                account_handle.set_client_options(*options.clone()).await?;
                Ok(ResponseType::UpdatedClientOptions)
            }
            AccountMethod::GetNodeInfo(url, jwt, auth) => {
                let auth = auth.as_ref().map(|a| (a.0.as_str(), a.1.as_str()));

                let info = account_handle
                    .get_node_info(url.as_deref(), jwt.as_deref(), auth)
                    .await?;
                Ok(ResponseType::NodeInfo(info))
            }
        }
    }

    /// The remove account message handler.
    async fn remove_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
        self.account_manager
            .remove_account(account_id.clone())
            .await
            .map(|_| ResponseType::RemovedAccount(account_id.clone()))
    }

    /// The create account message handler.
    async fn create_account(&self, account: &AccountToCreate) -> Result<ResponseType> {
        let mut builder = self.account_manager.create_account(account.client_options.clone())?;

        if let Some(alias) = &account.alias {
            builder = builder.alias(alias);
        }
        if let Some(created_at) = &account.created_at {
            builder = builder.created_at(*created_at);
        }
        if account.skip_persistence {
            builder = builder.skip_persistence();
        }
        if let Some(signer_type) = &account.signer_type {
            builder = builder.signer_type(signer_type.clone());
        }
        if account.allow_create_multiple_empty_accounts {
            builder = builder.allow_create_multiple_empty_accounts();
        }

        match builder.initialise().await {
            Ok(account_handle) => {
                let account = account_handle.read().await;
                Ok(ResponseType::CreatedAccount(AccountDto::new(
                    account.clone(),
                    Vec::new(),
                )))
            }
            Err(e) => Err(e),
        }
    }

    async fn get_account(&self, account_id: &AccountIdentifier) -> Result<ResponseType> {
        let account_handle = self.account_manager.get_account(account_id.clone()).await?;
        let account = account_handle.read().await;
        let messages = account.list_messages(0, 0, None).await?;
        Ok(ResponseType::ReadAccount(AccountDto::new(account.clone(), messages)))
    }

    async fn get_accounts(&self) -> Result<ResponseType> {
        let accounts = self.account_manager.get_accounts().await?;
        let mut accounts_ = Vec::new();
        for account_handle in accounts {
            let account = account_handle.read().await;
            let messages = account.list_messages(0, 0, None).await?;
            accounts_.push(AccountDto::new(account.clone(), messages));
        }
        Ok(ResponseType::ReadAccounts(accounts_))
    }

    async fn set_storage_password(&self, password: &str) -> Result<ResponseType> {
        self.account_manager.set_storage_password(password).await?;
        Ok(ResponseType::StoragePasswordSet)
    }

    #[cfg(feature = "stronghold")]
    async fn set_stronghold_password(&self, password: &str) -> Result<ResponseType> {
        self.account_manager.set_stronghold_password(password).await?;
        Ok(ResponseType::StrongholdPasswordSet)
    }

    async fn send_transfer(&self, account_id: &AccountIdentifier, transfer: Transfer) -> Result<ResponseType> {
        let account = self.account_manager.get_account(account_id.clone()).await?;
        let message = account.transfer(transfer).await?;
        Ok(ResponseType::SentTransfer(message))
    }

    async fn internal_transfer(
        &self,
        from_account_id: &AccountIdentifier,
        to_account_id: &AccountIdentifier,
        amount: NonZeroU64,
    ) -> Result<ResponseType> {
        let message = self
            .account_manager
            .internal_transfer(from_account_id.clone(), to_account_id.clone(), amount)
            .await?;
        Ok(ResponseType::SentTransfer(message))
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountToCreate, Message, MessageType, Response, ResponseType, WalletMessageHandler};
    use crate::{account_manager::AccountManager, client::ClientOptionsBuilder};
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

    /// The wallet actor builder.
    #[derive(Default)]
    pub struct WalletBuilder {
        rx: Option<UnboundedReceiver<Message>>,
        message_handler: Option<WalletMessageHandler>,
    }

    impl WalletBuilder {
        /// Creates a new wallet actor builder.
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets the receiver for messages.
        pub fn rx(mut self, rx: UnboundedReceiver<Message>) -> Self {
            self.rx.replace(rx);
            self
        }

        /// Sets the wallet message handler
        pub fn message_handler(mut self, message_handler: WalletMessageHandler) -> Self {
            self.message_handler.replace(message_handler);
            self
        }

        /// Builds the Wallet actor.
        pub async fn build(self) -> Wallet {
            Wallet {
                rx: self.rx.expect("rx is required"),
                message_handler: self.message_handler.expect("message handler is required"),
            }
        }
    }

    /// The Account actor.
    pub struct Wallet {
        rx: UnboundedReceiver<Message>,
        message_handler: WalletMessageHandler,
    }

    impl Wallet {
        /// Runs the actor.
        pub async fn run(mut self) {
            println!("running wallet actor");

            while let Some(message) = self.rx.recv().await {
                self.message_handler.handle(message).await;
            }
        }
    }

    fn spawn_actor(manager: AccountManager) -> UnboundedSender<Message> {
        let (tx, rx) = unbounded_channel();
        std::thread::spawn(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                let actor = WalletBuilder::new()
                    .rx(rx)
                    .message_handler(WalletMessageHandler::with_manager(manager))
                    .build()
                    .await;
                actor.run().await
            });
        });
        tx
    }

    async fn send_message(tx: &UnboundedSender<Message>, message_type: MessageType) -> Response {
        let (message_tx, mut message_rx) = unbounded_channel();
        let message = Message::new("".to_string(), message_type, message_tx);
        tx.send(message).unwrap();
        message_rx.recv().await.unwrap()
    }

    #[tokio::test]
    async fn create_and_remove_account() {
        crate::test_utils::with_account_manager(
            crate::test_utils::TestType::SigningAndStorage,
            |manager, signer_type| async move {
                let tx = spawn_actor(manager);

                // create an account
                let account = AccountToCreate {
                    client_options: ClientOptionsBuilder::new()
                        .with_node("https://api.thin-hornet-0.h.chrysalis-devnet.iota.cafe/")
                        .unwrap()
                        .build()
                        .unwrap(),
                    alias: None,
                    created_at: None,
                    skip_persistence: false,
                    signer_type: Some(signer_type.clone()),
                    allow_create_multiple_empty_accounts: false,
                };
                #[cfg(feature = "stronghold")]
                send_message(&tx, MessageType::SetStrongholdPassword("password".to_string())).await;
                send_message(
                    &tx,
                    MessageType::StoreMnemonic {
                        signer_type,
                        mnemonic: None,
                    },
                )
                .await;
                let response = send_message(&tx, MessageType::CreateAccount(Box::new(account))).await;
                match response.response() {
                    ResponseType::CreatedAccount(created_account) => {
                        let id = created_account.account.id().clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(6));
                            // remove the created account
                            let response = crate::block_on(async move {
                                send_message(&tx, MessageType::RemoveAccount(id.into())).await
                            });
                            assert!(matches!(response.response(), ResponseType::RemovedAccount(_)));
                        });
                    }
                    _ => panic!("unexpected response {:?}", response),
                }
            },
        )
        .await;
    }

    #[tokio::test]
    async fn legacy_address_checksum() {
        crate::test_utils::with_account_manager(crate::test_utils::TestType::Signing, |manager, _| async move {
            let tx = spawn_actor(manager);
            let response = send_message(
                &tx,
                MessageType::GetLegacyAddressChecksum(
                    "I9HZLJSWABQNFGUZQUETRIUAERKZZXSPGRWXZWPMQDWLIMHSNCMDKIOEVQBKTDBCDNYDOHAYOVBYJYEEY".to_string(),
                ),
            )
            .await;
            match response.response() {
                ResponseType::GetLegacyAddressChecksum(address) => {
                    assert_eq!(
                        address,
                        &"I9HZLJSWABQNFGUZQUETRIUAERKZZXSPGRWXZWPMQDWLIMHSNCMDKIOEVQBKTDBCDNYDOHAYOVBYJYEEYPDKGK9VPZ"
                            .to_string()
                    );
                }
                _ => panic!("unexpected response {:?}", response),
            }
        })
        .await;
    }
}
