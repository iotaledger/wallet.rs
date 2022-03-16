// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountHandle, Error, Result};

use std::time::{SystemTime, UNIX_EPOCH};

/// Max allowed difference between the local time and latest milestone time, 5 minutes in seconds
const FIVE_MINUTES_IN_SECONDS: u64 = 300;

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
