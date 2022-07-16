// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::output::Output;

use crate::account::types::{AddressWithUnspentOutputs, OutputData};

// Check if an output has an expired ExpirationUnlockCondition
pub(crate) fn is_expired(output: &Output, current_time: u32) -> bool {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if let Some(expiration) = unlock_conditions.expiration() {
            expiration.timestamp() < current_time
        } else {
            false
        }
    } else {
        false
    }
}

// Check if an output can be unlocked by one of the account addresses at the current time
pub(crate) fn can_output_be_unlocked_now(
    // We use the addresses with unspent outputs, because other addresses of the account without unspent outputs can't
    // be related to this output
    account_addresses: &[AddressWithUnspentOutputs],
    output_data: &OutputData,
    current_time: u32,
) -> bool {
    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
        if unlock_conditions.is_time_locked(current_time) {
            return false;
        }

        let output_address = unlock_conditions
            .address()
            .expect("Output needs to have an address unlock condition")
            .address();

        let unlock_address = unlock_conditions.locked_address(output_address, current_time);
        // The address that can unlock the output needs to belong to the account
        if !account_addresses.iter().any(|a| a.address.inner == *unlock_address) {
            return false;
        };

        true
    } else {
        false
    }
}

// Check if an output can be unlocked by one of the account addresses at the current time and at any
// point in the future
pub(crate) fn can_output_be_unlocked_forever_from_now_on(
    // We use the addresses with unspent outputs, because other addresses of the account without unspent outputs can't
    // be related to this output
    account_addresses: &[AddressWithUnspentOutputs],
    output_data: &OutputData,
    current_time: u32,
) -> bool {
    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
        if unlock_conditions.is_time_locked(current_time) {
            return false;
        }

        // If there is an expiration unlock condition, we can only unlock it forever from now on, if it's expired and
        // the return address belongs to the account
        if let Some(expiration) = unlock_conditions.expiration() {
            if let Some(return_address) = expiration.return_address_expired(current_time) {
                if !account_addresses.iter().any(|a| a.address.inner == *return_address) {
                    return false;
                };
            } else {
                return false;
            }
        }

        true
    } else {
        false
    }
}
