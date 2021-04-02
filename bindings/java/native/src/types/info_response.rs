// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::{CopyGetters, Getters};
use iota::bee_rest_api::types::responses::InfoResponse as RustInfoResponse;

#[derive(PartialEq, Getters, CopyGetters)]
pub struct InfoResponse {
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    version: String,
    #[getset(get_copy = "pub")]
    is_healthy: bool,
    #[getset(get = "pub")]
    network_id: String,
    #[getset(get = "pub")]
    bech32_hrp: String,
    #[getset(get_copy = "pub")]
    latest_milestone_index: u32,
    #[getset(get_copy = "pub")]
    confirmed_milestone_index: u32,
    #[getset(get_copy = "pub")]
    pruning_index: u32,
    features: Vec<String>,
    #[getset(get_copy = "pub")]
    min_pow_score: f64,
}

impl InfoResponse {
    pub fn features(&self) -> Vec<String> {
        self.features.to_vec()
    }
}

impl From<RustInfoResponse> for InfoResponse {
    fn from(info: RustInfoResponse) -> Self {
        Self {
            name: info.name,
            version: info.version,
            is_healthy: info.is_healthy,
            network_id: info.network_id,
            bech32_hrp: info.bech32_hrp,
            latest_milestone_index: info.latest_milestone_index,
            confirmed_milestone_index: info.confirmed_milestone_index,
            pruning_index: info.pruning_index,
            features: info.features,
            min_pow_score: info.min_pow_score,
        }
    }
}
