use crate::address::Address;
use crate::message::Message;
use iota::transaction::prelude::MessageId;

/// Monitor address for balance changes.
pub fn on_address_balance_change<F: Fn((Address, u64))>(cb: F) {}

/// Monitor address for new messages.
pub fn on_address_new_transaction<F: Fn((Address, MessageId))>(cb: F) {}

/// Monitor transaction for confirmation state.
pub fn on_confirmation_state_change<F: Fn((Message, bool))>(cb: F) {}
