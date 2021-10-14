// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::{CopyGetters, Getters};
use std::{path::PathBuf, time::Duration};

use crate::Result;
use anyhow::anyhow;

use iota_wallet::{
    event::{
        AddressConsolidationNeeded as WalletAddressConsolidationNeeded, BalanceEvent as WalletBalanceEvent, EventId,
        MigrationProgressType as WalletMigrationProgressType,
        TransferProgressType as WalletTransferProgressType,
        TransactionConfirmationChangeEvent as WalletTransactionConfirmationChangeEvent,
        TransactionEvent as WalletTransactionEvent, TransactionReattachmentEvent as WalletTransactionReattachmentEvent,
        TransferProgress as WalletTransferProgress,
        AddressData as WalletAddressData,
        PreparedTransactionData as WalletPreparedTransactionData,
        TransactionIO as WalletTransactionIO
    },
    StrongholdSnapshotStatus as SnapshotStatus, StrongholdStatus as StrongholdStatusWallet,
};

pub struct EventManager {}

#[derive(Copy, Clone)]
pub enum MigrationProgressType {
    FetchingMigrationData = 0,
    MiningBundle = 1,
    SigningBundle = 2,
    BroadcastingBundle = 3,
    TransactionConfirmed = 4,
}

pub fn migration_progress_type_enum_to_type(migration_type: &WalletMigrationProgressType) -> MigrationProgressType {
    match migration_type {
        WalletMigrationProgressType::FetchingMigrationData { .. } => MigrationProgressType::FetchingMigrationData,
        WalletMigrationProgressType::MiningBundle { .. } => MigrationProgressType::MiningBundle,
        WalletMigrationProgressType::SigningBundle { .. } => MigrationProgressType::SigningBundle,
        WalletMigrationProgressType::BroadcastingBundle { .. } => MigrationProgressType::BroadcastingBundle,
        WalletMigrationProgressType::TransactionConfirmed { .. } => MigrationProgressType::TransactionConfirmed,
    }
}

#[derive(Copy, Clone)]
pub enum TransferProgressType {
    SyncingAccount = 0,
    SelectingInputs = 1,
    GeneratingRemainderDepositAddress = 2,
    PreparedTransaction = 3,
    SigningTransaction = 4,
    PerformingPoW = 5,
    Broadcasting = 6,
}

pub fn transfer_progress_type_enum_to_type(transfer_type: &WalletTransferProgressType) -> TransferProgressType {
    match transfer_type {
        WalletTransferProgressType::SyncingAccount { .. } => TransferProgressType::SyncingAccount,
        WalletTransferProgressType::SelectingInputs { .. } => TransferProgressType::SelectingInputs,
        WalletTransferProgressType::GeneratingRemainderDepositAddress { .. } => TransferProgressType::GeneratingRemainderDepositAddress,
        WalletTransferProgressType::PreparedTransaction { .. } => TransferProgressType::PreparedTransaction,
        WalletTransferProgressType::SigningTransaction { .. } => TransferProgressType::SigningTransaction,
        WalletTransferProgressType::PerformingPoW { .. } => TransferProgressType::PerformingPoW,
        WalletTransferProgressType::Broadcasting { .. } => TransferProgressType::Broadcasting,
    }
}

#[derive(Getters, CopyGetters)]
pub struct FetchingMigrationData {
    #[getset(get_copy = "pub")]
    initial_address_index: u64,
    #[getset(get_copy = "pub")]
    final_address_index: u64,
}

#[derive(Getters, CopyGetters)]
pub struct MiningBundle {
    #[getset(get = "pub")]
    address: String,
}

pub struct SigningBundle {
    addresses: Vec<String>,
}
impl SigningBundle {
    pub fn addresses(&self) -> Vec<String> {
        self.addresses.clone()
    }
}

#[derive(Getters, CopyGetters)]
pub struct BroadcastingBundle {
    #[getset(get = "pub")]
    bundle_hash: String,
}

#[derive(Getters, CopyGetters)]
pub struct TransactionConfirmed {
    #[getset(get = "pub")]
    bundle_hash: String,
}

pub enum StrongholdStatusType {
    Unlocked = 0,
    Locked = 1,
}

pub struct StrongholdStatusEvent {
    status: StrongholdStatusWallet,
}

impl StrongholdStatusEvent {
    pub fn snapshot_path(&self) -> PathBuf {
        self.status.snapshot_path().clone()
    }
    pub fn status(&self) -> StrongholdStatusType {
        match self.status.snapshot() {
            SnapshotStatus::Locked => StrongholdStatusType::Locked,
            SnapshotStatus::Unlocked(_) => StrongholdStatusType::Unlocked,
        }
    }
    pub fn unlocked_duration(&self) -> Result<Duration> {
        match self.status.snapshot() {
            SnapshotStatus::Locked => Err(anyhow!("Stronghold is locked")),
            SnapshotStatus::Unlocked(d) => Ok(*d),
        }
    }
}

pub struct TransferProgress {
    transfer_type: TransferProgressType,
    event: WalletTransferProgressType,
}

impl TransferProgress {
    pub fn get_type(&self) -> TransferProgressType {
        self.transfer_type
    }

    pub fn as_prepared_transaction(&self) -> Result<PreparedTransactionData> {
        if let WalletTransferProgressType::PreparedTransaction(data) = &self.event {
            Ok(data.into())
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }

    pub fn as_generating_remainder_deposit_address(&self) -> Result<AddressData> {
        if let WalletTransferProgressType::GeneratingRemainderDepositAddress(data) = &self.event {
            Ok(data.into())
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }
}

/// Address event data.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct AddressData {
    /// The address.
    #[getset(get = "pub")]
    pub address: String,
}

impl From<&WalletAddressData> for AddressData {
    fn from(data: &WalletAddressData) -> Self {
        Self {
            address: data.address().clone(),
        }
    }
}
impl core::fmt::Display for AddressData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "address={:?}", self.address)
    }
}

/// Prepared transaction event data.
#[derive(Clone, Debug)]
pub struct PreparedTransactionData {
    pub inputs: Vec<TransactionIO>,
    pub outputs: Vec<TransactionIO>,
    pub data: Option<String>,
}

impl From<&WalletPreparedTransactionData> for PreparedTransactionData {
    fn from(data: &WalletPreparedTransactionData) -> Self {
        Self {
            inputs: data.inputs().clone().iter().map(|d| d.into()).collect(),
            outputs: data.inputs().clone().iter().map(|d| d.into()).collect(),
            data: data.data().clone(),
        }
    }
}
impl core::fmt::Display for PreparedTransactionData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "data={:?}, inputs=({:?}), outputs=({:?})", self.data, self.inputs, self.outputs)
    }
}

impl PreparedTransactionData {
    pub fn inputs(&self) -> Vec<TransactionIO> {
        self.inputs.clone()
    } 
    pub fn outputs(&self) -> Vec<TransactionIO> {
        self.outputs.clone()
    }
    pub fn data(&self) -> Option<String> {
        self.data.clone()
    }
}

/// Input or output data for PreparedTransactionData
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct TransactionIO {
    #[getset(get = "pub")]
    pub address: String,
    #[getset(get_copy = "pub")]
    pub amount: u64,
    #[getset(get_copy = "pub")]
    pub remainder: Option<bool>,
}

impl From<&WalletTransactionIO> for TransactionIO {
    fn from(data: &WalletTransactionIO) -> Self {
        Self {
            address: data.address().clone(),
            amount: data.amount().clone(),
            remainder: data.remainder().clone(),
        }
    }
}
impl core::fmt::Display for TransactionIO {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "address={:?}, amount={:?}, remainder={:?}", self.address, self.amount, self.remainder)
    }
}

impl From<WalletTransferProgressType> for TransferProgress {
    fn from(progress_type: WalletTransferProgressType) -> Self {
        Self { 
            transfer_type: transfer_progress_type_enum_to_type(&progress_type),
            event: progress_type 
        }
    }
}

impl core::fmt::Display for TransferProgress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "type={:?}", self.event)
    }
}

pub struct MigrationProgressEvent {
    migration_type: MigrationProgressType,
    event: WalletMigrationProgressType,
}

impl MigrationProgressEvent {
    pub fn get_type(&self) -> MigrationProgressType {
        self.migration_type
    }

    pub fn as_fetching_migration_data(&self) -> Result<FetchingMigrationData> {
        if let WalletMigrationProgressType::FetchingMigrationData {
            initial_address_index,
            final_address_index,
        } = &self.event
        {
            Ok(FetchingMigrationData {
                initial_address_index: *initial_address_index,
                final_address_index: *final_address_index,
            })
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }
    pub fn as_mining_bundle(&self) -> Result<MiningBundle> {
        if let WalletMigrationProgressType::MiningBundle { address } = &self.event {
            Ok(MiningBundle {
                address: address.clone(),
            })
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }
    pub fn as_signing_bundle(&self) -> Result<SigningBundle> {
        if let WalletMigrationProgressType::SigningBundle { addresses } = &self.event {
            Ok(SigningBundle {
                addresses: addresses.clone(),
            })
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }
    pub fn as_broadcasting_bundle(&self) -> Result<BroadcastingBundle> {
        if let WalletMigrationProgressType::BroadcastingBundle { bundle_hash } = &self.event {
            Ok(BroadcastingBundle {
                bundle_hash: bundle_hash.clone(),
            })
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }
    pub fn as_transaction_confirmed(&self) -> Result<TransactionConfirmed> {
        if let WalletMigrationProgressType::TransactionConfirmed { bundle_hash } = &self.event {
            Ok(TransactionConfirmed {
                bundle_hash: bundle_hash.clone(),
            })
        } else {
            Err(anyhow!("wrong migration type"))
        }
    }
}

impl core::fmt::Display for MigrationProgressEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "type={:?}", self.event)
    }
}

pub trait ErrorListener {
    fn on_error(&self, error: String);
}

pub trait NewTransactionListener {
    fn on_new_transaction(&self, event: WalletTransactionEvent);
}

pub trait ReattachTransactionListener {
    fn on_reattachment(&self, event: WalletTransactionReattachmentEvent);
}

pub trait BroadcastTransactionListener {
    fn on_broadcast(&self, event: WalletTransactionEvent);
}

pub trait TransactionConfirmationChangeListener {
    fn on_confirmation_state_change(&self, event: WalletTransactionConfirmationChangeEvent);
}

pub trait TransferProgressListener {
    fn on_transfer_progress(&self, event: WalletTransferProgress);
}

pub trait MigrationProgressListener {
    fn on_migration_progress(&self, event: MigrationProgressEvent);
}

pub trait BalanceChangeListener {
    fn on_balance_change(&self, event: WalletBalanceEvent);
}

// Ledger
pub trait AddressConsolidationNeededListener {
    fn on_address_consolidation_needed(&self, event: WalletAddressConsolidationNeeded);
}

// Stronghold
pub trait StrongholdStatusListener {
    fn on_stronghold_status_change(&self, event: StrongholdStatusEvent);
}

impl EventManager {
    pub fn subscribe_new_transaction(cb: Box<dyn NewTransactionListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_new_transaction(move |event| {
                cb.on_new_transaction(event.clone());
            })
            .await
        })
    }

    pub fn remove_new_transaction_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_new_transaction_listener(&event).await })
    }

    pub fn subscribe_confirmation_state_change(
        cb: Box<dyn TransactionConfirmationChangeListener + Send + 'static>,
    ) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_confirmation_state_change(move |event| {
                cb.on_confirmation_state_change(event.clone());
            })
            .await
        })
    }

    pub fn remove_confirmation_state_change_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_confirmation_state_change_listener(&event).await })
    }

    pub fn subscribe_reattachment(cb: Box<dyn ReattachTransactionListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_reattachment(move |event| {
                cb.on_reattachment(event.clone());
            })
            .await
        })
    }

    pub fn remove_reattachment_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_reattachment_listener(&event).await })
    }

    pub fn subscribe_broadcast(cb: Box<dyn BroadcastTransactionListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_broadcast(move |event| {
                cb.on_broadcast(event.clone());
            })
            .await
        })
    }

    pub fn remove_broadcast_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_broadcast_listener(&event).await })
    }

    pub fn subscribe_transfer_progress(cb: Box<dyn TransferProgressListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_transfer_progress(move |event| {
                cb.on_transfer_progress(event.clone());
            })
            .await
        })
    }

    pub fn remove_transfer_progress_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_transfer_progress_listener(&event).await })
    }

    pub fn subscribe_migration_progress(cb: Box<dyn MigrationProgressListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_migration_progress(move |event| {
                cb.on_migration_progress(MigrationProgressEvent {
                    migration_type: migration_progress_type_enum_to_type(&event.event),
                    event: event.event.clone(),
                });
            })
            .await
        })
    }

    pub fn remove_migration_progress_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_migration_progress_listener(&event).await })
    }

    pub fn subscribe_balance_change(cb: Box<dyn BalanceChangeListener + Send + 'static>) -> EventId {
        crate::block_on(async move {
            iota_wallet::event::on_balance_change(move |event| {
                cb.on_balance_change(event.clone());
            })
            .await
        })
    }

    pub fn remove_balance_change_listener(event: EventId) {
        crate::block_on(async move { iota_wallet::event::remove_balance_change_listener(&event).await })
    }

    pub fn subscribe_errors(cb: Box<dyn ErrorListener + Send + 'static>) -> EventId {
        iota_wallet::event::on_error(move |error| {
            cb.on_error(error.to_string());
        })
    }

    pub fn remove_error_listener(event: EventId) {
        iota_wallet::event::remove_error_listener(&event)
    }

    #[allow(unreachable_code)]
    pub fn subscribe_stronghold_status_change(
        cb: Box<dyn StrongholdStatusListener + Send + 'static>,
    ) -> Result<EventId> {
        #[cfg(feature = "stronghold")]
        {
            let id = crate::block_on(async move {
                iota_wallet::event::on_stronghold_status_change(move |event| {
                    cb.on_stronghold_status_change(StrongholdStatusEvent { status: event.clone() });
                })
                .await
            });
            return Ok(id);
        }
        Err(anyhow!("No stronghold found during compilation"))
    }

    pub fn remove_stronghold_status_change_listener(event: EventId) {
        #[cfg(feature = "stronghold")]
        {
            crate::block_on(async move { iota_wallet::event::remove_stronghold_status_change_listener(&event).await })
        }
    }

    #[allow(unreachable_code)]
    pub fn subscribe_address_consolidation_needed(
        _cb: Box<dyn AddressConsolidationNeededListener + Send + 'static>,
    ) -> Result<EventId> {
        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
        {
            let id: EventId = crate::block_on(async move {
                iota_wallet::event::on_address_consolidation_needed(move |event| {
                    _cb.on_address_consolidation_needed(event.clone());
                })
                .await
            });
            return Ok(id);
        }

        Err(anyhow!("No ledger found during compilation"))
    }

    pub fn remove_address_consolidation_needed_listener(_event: EventId) {
        #[cfg(any(feature = "ledger-nano", feature = "ledger-nano-simulator"))]
        {
            crate::block_on(
                async move { iota_wallet::event::remove_address_consolidation_needed_listener(&_event).await },
            )
        }
    }
}
