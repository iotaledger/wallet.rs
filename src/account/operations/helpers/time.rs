// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::block::address::Address;

use crate::account::types::{AddressWithUnspentOutputs, OutputData};

// Check if an output can be unlocked by one of the account addresses at the current time
pub(crate) fn can_output_be_unlocked_now(
    // We use the addresses with unspent outputs, because other addresses of the account without unspent outputs can't
    // be related to this output
    account_addresses: &[AddressWithUnspentOutputs],
    alias_and_nft_addresses: &[Address],
    output_data: &OutputData,
    current_time: u32,
    alias_state_transition: bool,
) -> crate::Result<bool> {
    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
        if unlock_conditions.is_time_locked(current_time) {
            return Ok(false);
        }
    }

    let (required_unlock_address, _unlocked_alias_or_nft_address) = output_data.output.required_and_unlocked_address(
        current_time,
        output_data.output_id,
        alias_state_transition,
    )?;

    Ok(account_addresses
        .iter()
        .any(|a| a.address.inner == required_unlock_address)
        || alias_and_nft_addresses.iter().any(|a| *a == required_unlock_address))
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
