// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use getset::{CopyGetters, Getters};
use iota_wallet::iota_client::{
    api_types::responses::InfoResponse as RustInfoResponse, NodeInfoWrapper as RustNodeInfoWrapper,
};

#[derive(PartialEq, Getters, CopyGetters)]
pub struct NodeInfoWrapper {
    #[getset(get = "pub")]
    url: String,
    node_info: InfoResponse,
}

impl NodeInfoWrapper {
    pub fn node_info(&self) -> InfoResponse {
        self.node_info.clone()
    }
}

impl From<RustNodeInfoWrapper> for NodeInfoWrapper {
    fn from(info: RustNodeInfoWrapper) -> Self {
        Self {
            url: info.url,
            node_info: InfoResponse {
                name: info.node_info.name,
                version: info.node_info.version,
                is_healthy: info.node_info.is_healthy,
                network_id: info.node_info.network_id,
                bech32_hrp: info.node_info.bech32_hrp,
                min_pow_score: info.node_info.min_pow_score,
                messages_per_second: info.node_info.messages_per_second,
                referenced_messages_per_second: info.node_info.referenced_messages_per_second,
                referenced_rate: info.node_info.referenced_rate,
                latest_milestone_timestamp: info.node_info.latest_milestone_timestamp,
                latest_milestone_index: info.node_info.latest_milestone_index,
                confirmed_milestone_index: info.node_info.confirmed_milestone_index,
                pruning_index: info.node_info.pruning_index,
                features: info.node_info.features,
            },
        }
    }
}

#[derive(Clone, PartialEq, Getters, CopyGetters)]
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
    min_pow_score: f64,
    #[getset(get_copy = "pub")]
    messages_per_second: f64,
    #[getset(get_copy = "pub")]
    referenced_messages_per_second: f64,
    #[getset(get_copy = "pub")]
    referenced_rate: f64,
    #[getset(get_copy = "pub")]
    latest_milestone_timestamp: u64,
    #[getset(get_copy = "pub")]
    latest_milestone_index: u32,
    #[getset(get_copy = "pub")]
    confirmed_milestone_index: u32,
    #[getset(get_copy = "pub")]
    pruning_index: u32,
    features: Vec<String>,
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
            messages_per_second: info.messages_per_second,
            referenced_messages_per_second: info.referenced_messages_per_second,
            referenced_rate: info.referenced_rate,
            latest_milestone_timestamp: info.latest_milestone_timestamp,
            latest_milestone_index: info.latest_milestone_index,
            confirmed_milestone_index: info.confirmed_milestone_index,
            pruning_index: info.pruning_index,
            features: info.features,
            min_pow_score: info.min_pow_score,
        }
    }
}
