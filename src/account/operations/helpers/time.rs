// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::bee_block::output::{
    unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition},
    Output, UnlockCondition,
};

use crate::account::types::{AddressWithUnspentOutputs, OutputData};

// Check if an output has an expired ExpirationUnlockCondition
pub(crate) fn is_expired(output: &Output, current_time: u32, current_milestone: u32) -> bool {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if let Some(UnlockCondition::Expiration(expiration)) = unlock_conditions.get(ExpirationUnlockCondition::KIND) {
            let mut ms_expired = false;
            let mut time_expired = false;
            // 0 gets ignored
            if *expiration.milestone_index() == 0 || *expiration.milestone_index() < current_milestone {
                ms_expired = true;
            }
            if expiration.timestamp() == 0 || expiration.timestamp() < current_time {
                time_expired = true;
            }
            // Check if the address which can unlock the output now is in the account
            ms_expired && time_expired
        } else {
            false
        }
    } else {
        false
    }
}

// Check if an output can be unlocked by one of the account addresses at the current time/milestone index
pub(crate) fn can_output_be_unlocked_now(
    // We use the addresses with unspent outputs, because other addresses of the account without unspent outputs can't
    // be related to this output
    account_addresses: &[AddressWithUnspentOutputs],
    output_data: &OutputData,
    current_time: u32,
    current_milestone: u32,
) -> bool {
    let mut can_be_unlocked = Vec::new();
    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
        for unlock_condition in unlock_conditions.iter() {
            match unlock_condition {
                UnlockCondition::Expiration(expiration) => {
                    let mut ms_expired = false;
                    let mut time_expired = false;
                    // 0 gets ignored
                    if *expiration.milestone_index() == 0 || *expiration.milestone_index() <= current_milestone {
                        ms_expired = true;
                    }
                    if expiration.timestamp() == 0 || expiration.timestamp() <= current_time {
                        time_expired = true;
                    }
                    // Check if the address which can unlock the output now is in the account
                    if ms_expired && time_expired {
                        // compare return address with associated address first, but if that doesn't match we also need
                        // to check all account addresses, because the associated address
                        // can only be the unlock address or the storage deposit address and not both (unless they're
                        // the same, which would mean transaction to oneself)
                        can_be_unlocked.push(
                            output_data.address == *expiration.return_address()
                                || account_addresses
                                    .iter()
                                    .any(|a| a.address.inner == *expiration.return_address()),
                        );
                        // Only if both conditions aren't met, the target address can unlock the output. If only one
                        // condition is met, the output can't be unlocked by anyone.
                    } else if (*expiration.milestone_index() == 0 || *expiration.milestone_index() > current_milestone)
                        && (expiration.timestamp() == 0 || expiration.timestamp() > current_time)
                    {
                        // check address unlock condition
                        let can_unlocked = if let Some(UnlockCondition::Address(address_unlock_condition)) =
                            unlock_conditions.get(AddressUnlockCondition::KIND)
                        {
                            // compare address_unlock_condition address with associated address first, but if that
                            // doesn't match we also need to check all account addresses,
                            // because the associated address can only be the unlock address
                            // or the storage deposit address and not both (unless they're
                            // the same, which would mean transaction to oneself)
                            output_data.address == *address_unlock_condition.address()
                                || account_addresses
                                    .iter()
                                    .any(|a| a.address.inner == *address_unlock_condition.address())
                        } else {
                            false
                        };
                        can_be_unlocked.push(can_unlocked);
                    } else {
                        can_be_unlocked.push(false);
                    }
                }
                UnlockCondition::Timelock(timelock) => {
                    let mut ms_reached = false;
                    let mut time_reached = false;
                    // 0 gets ignored
                    if *timelock.milestone_index() == 0 || *timelock.milestone_index() < current_milestone {
                        ms_reached = true;
                    }
                    if timelock.timestamp() == 0 || timelock.timestamp() < current_time {
                        time_reached = true;
                    }
                    can_be_unlocked.push(ms_reached && time_reached);
                }
                _ => {}
            }
        }
        // If one of the unlock conditions is not met, then we can't unlock it
        !can_be_unlocked.contains(&false)
    } else {
        false
    }
}

// Check if an output can be unlocked by one of the account addresses at the current time/milestone index and at any
// point in the future
pub(crate) fn can_output_be_unlocked_forever_from_now_on(
    // We use the addresses with unspent outputs, because other addresses of the account without unspent outputs can't
    // be related to this output
    account_addresses: &[AddressWithUnspentOutputs],
    output_data: &OutputData,
    current_time: u32,
    current_milestone: u32,
) -> bool {
    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
        for unlock_condition in unlock_conditions.iter() {
            match unlock_condition {
                UnlockCondition::Expiration(expiration) => {
                    let mut ms_expired = false;
                    let mut time_expired = false;
                    // 0 gets ignored
                    if *expiration.milestone_index() == 0 || *expiration.milestone_index() <= current_milestone {
                        ms_expired = true;
                    }
                    if expiration.timestamp() == 0 || expiration.timestamp() <= current_time {
                        time_expired = true;
                    }
                    // Check if the address which can unlock the output now is in the account
                    if ms_expired && time_expired {
                        // compare return address with the account addresses
                        if !account_addresses
                            .iter()
                            .any(|a| a.address.inner == *expiration.return_address())
                        {
                            return false;
                        };
                    } else {
                        return false;
                    }
                }
                UnlockCondition::Timelock(timelock) => {
                    let mut ms_reached = false;
                    let mut time_reached = false;
                    // 0 gets ignored
                    if *timelock.milestone_index() == 0 || *timelock.milestone_index() <= current_milestone {
                        ms_reached = true;
                    }
                    if timelock.timestamp() == 0 || timelock.timestamp() <= current_time {
                        time_reached = true;
                    }
                    if !(ms_reached && time_reached) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    } else {
        false
    }
}
