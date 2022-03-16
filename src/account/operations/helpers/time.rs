// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{constants::FIVE_MINUTES_IN_SECONDS, types::OutputData, AccountHandle},
    Error, Result,
};

use iota_client::bee_message::output::{
    unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition},
    Output, UnlockCondition,
};

use std::time::{SystemTime, UNIX_EPOCH};

impl AccountHandle {
    /// Get the local time, but compare it to the time from the nodeinfo, if it's off more than 5 minutes, an error will
    /// be returned
    pub(crate) async fn get_time_and_milestone_checked(&self) -> Result<(u64, u32)> {
        let local_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let status_response = self.client.get_info().await?.nodeinfo.status;
        let latest_ms_timestamp = status_response.latest_milestone_timestamp;
        // Check the local time is in the range of +-5 minutes of the node to prevent locking funds by accident
        if !(latest_ms_timestamp - FIVE_MINUTES_IN_SECONDS..latest_ms_timestamp + FIVE_MINUTES_IN_SECONDS)
            .contains(&local_time)
        {
            return Err(Error::TimeNotSynced(local_time, latest_ms_timestamp));
        }
        Ok((local_time, status_response.latest_milestone_index))
    }
}

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
pub(crate) fn can_output_be_unlocked_now(output_data: &OutputData, current_time: u32, current_milestone: u32) -> bool {
    let mut can_be_unlocked = Vec::new();
    if let Some(unlock_conditions) = output_data.output.unlock_conditions() {
        for unlock_condition in unlock_conditions.iter() {
            match unlock_condition {
                UnlockCondition::Expiration(expiration) => {
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
                    if ms_expired && time_expired {
                        // check return address
                        can_be_unlocked.push(output_data.address == *expiration.return_address());
                    } else {
                        // check address unlock condition
                        let can_unlocked = if let Some(UnlockCondition::Address(address_unlock_condition)) =
                            unlock_conditions.get(AddressUnlockCondition::KIND)
                        {
                            output_data.address == *address_unlock_condition.address()
                        } else {
                            false
                        };
                        can_be_unlocked.push(can_unlocked);
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
