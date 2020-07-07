use crate::address::Address;
use crate::transaction::Transaction;
use bee_crypto::ternary::Hash;

/// Monitor address for balance changes.
pub fn on_address_balance_change<F: Fn((Address, u64))>(cb: F) {}

/// Monitor address for new transactions.
pub fn on_address_new_transaction<F: Fn((Address, Hash))>(cb: F) {}

/// Monitor transaction for confirmation state.
pub fn on_confirmation_state_change<F: Fn((Transaction, bool))>(cb: F) {}
