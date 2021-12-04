// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::participation::{
    response_types::{AddressStakingStatus, EventIds, EventInformation, EventStatus, OutputStatusResponse},
    types::ParticipationEventType,
};

use serde::{Deserialize, Serialize};

/// GET /api/plugins/participation/events : Lists all events, returning their EventID.
pub(crate) async fn get_events(
    mut node: iota_client::node_manager::Node,
    event_type: Option<ParticipationEventType>,
) -> crate::Result<EventIds> {
    if let Some(event_type) = event_type {
        let query_string = match event_type {
            ParticipationEventType::Voting => "0",
            ParticipationEventType::Staking => "1",
        };
        node.url.set_query(Some(query_string));
    }

    let path = "/api/plugins/participation/events";
    node.url.set_path(path);

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponseWrapper {
        data: EventIds,
    }
    let resp: ResponseWrapper = reqwest::get(node.url).await?.json().await?;
    Ok(resp.data)
}

// GET /api/plugins/participation/events/{eventID} : Returns the event information as a JSON payload.
pub(crate) async fn get_event_information(
    mut node: iota_client::node_manager::Node,
    event_id: &str,
) -> crate::Result<EventInformation> {
    let path = &format!("/api/plugins/participation/events/{}", event_id);
    node.url.set_path(path);

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponseWrapper {
        data: EventInformation,
    }
    let resp: ResponseWrapper = reqwest::get(node.url).await?.json().await?;
    Ok(resp.data)
}

// GET /api/plugins/participation/events/{eventID}/status : Returns the status of the given event
// (upcoming,commencing,holding,ended) and if it contains a Ballot, the current and accumulated answers for each
// question.
pub(crate) async fn get_event_status(
    mut node: iota_client::node_manager::Node,
    event_id: &str,
) -> crate::Result<EventStatus> {
    let path = &format!("/api/plugins/participation/events/{}/status", event_id);
    node.url.set_path(path);

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponseWrapper {
        data: EventStatus,
    }
    let resp: ResponseWrapper = reqwest::get(node.url).await?.json().await?;
    Ok(resp.data)
}

// GET /api/plugins/participation/addresses/{bech32address} : Returns the staking rewards as a JSON payload.
pub(crate) async fn get_address_staking_status(
    mut node: iota_client::node_manager::Node,
    address: String,
) -> crate::Result<AddressStakingStatus> {
    let path = &format!("/api/plugins/participation/addresses/{}", address);
    node.url.set_path(path);

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponseWrapper {
        data: AddressStakingStatus,
    }
    let resp: ResponseWrapper = reqwest::get(node.url).await?.json().await?;
    Ok(resp.data)
}

// GET /api/plugins/participation/outputs/{outputId} : Returns the amount and start milestone index for an output for
// staking as a JSON payload.
pub(crate) async fn get_output_participation(
    mut node: iota_client::node_manager::Node,
    output_id: String,
) -> crate::Result<OutputStatusResponse> {
    let path = &format!("/api/plugins/participation/outputs/{}", output_id);
    node.url.set_path(path);

    #[derive(Debug, Serialize, Deserialize)]
    struct ResponseWrapper {
        data: OutputStatusResponse,
    }
    let resp: ResponseWrapper = reqwest::get(node.url).await?.json().await?;
    Ok(resp.data)
}
