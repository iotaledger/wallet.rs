// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{error::Error, AddressWrapper, WalletMessage};

use dict_derive::{FromPyObject as DeriveFromPyObject, IntoPyObject as DeriveIntoPyObject};
use iota_wallet::event::{
    BalanceChange as WalletBalanceChange, BalanceEvent as WalletBalanceEvent,
    TransactionConfirmationChangeEvent as WalletTransactionConfirmationChangeEvent,
    TransactionEvent as WalletTransactionEvent, TransactionReattachmentEvent as WalletTransactionReattachmentEvent,
};

use std::convert::{TryFrom, TryInto};

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct BalanceChange {
    /// The change amount if it was a spent event.
    pub spent: u64,
    /// The change amount if it was a receive event.
    pub received: u64,
}

impl From<WalletBalanceChange> for BalanceChange {
    fn from(value: WalletBalanceChange) -> Self {
        Self {
            spent: value.spent,
            received: value.received,
        }
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct BalanceEvent {
    /// Associated account.
    account_id: String,
    /// The address.
    address: AddressWrapper,
    /// The balance change data.
    balance_change: BalanceChange,
}

impl TryFrom<WalletBalanceEvent> for BalanceEvent {
    type Error = Error;
    fn try_from(value: WalletBalanceEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: value.account_id,
            address: value.address.into(),
            balance_change: value.balance_change.into(),
        })
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct TransactionEvent {
    /// Associated account.
    account_id: String,
    /// The address.
    message: WalletMessage,
}

impl TryFrom<WalletTransactionEvent> for TransactionEvent {
    type Error = Error;
    fn try_from(value: WalletTransactionEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: value.account_id,
            message: value.message.try_into()?,
        })
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct TransactionConfirmationChangeEvent {
    /// Associated account.
    account_id: String,
    /// The associated message.
    message: WalletMessage,
    /// Confirmed flag.
    confirmed: bool,
}

impl TryFrom<WalletTransactionConfirmationChangeEvent> for TransactionConfirmationChangeEvent {
    type Error = Error;
    fn try_from(value: WalletTransactionConfirmationChangeEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: value.account_id,
            message: value.message.try_into()?,
            confirmed: value.confirmed,
        })
    }
}

#[derive(Debug, DeriveFromPyObject, DeriveIntoPyObject)]
pub struct TransactionReattachmentEvent {
    /// Associated account.
    account_id: String,
    /// The reattachment message.
    message: WalletMessage,
    /// The id of the message that was reattached.
    reattached_message_id: String,
}

impl TryFrom<WalletTransactionReattachmentEvent> for TransactionReattachmentEvent {
    type Error = Error;
    fn try_from(value: WalletTransactionReattachmentEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: value.account_id,
            message: value.message.try_into()?,
            reattached_message_id: value.reattached_message_id.to_string(),
        })
    }
}
