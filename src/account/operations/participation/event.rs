// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::{
    node_api::participation::types::{
        ParticipationEvent, ParticipationEventId, ParticipationEventStatus, ParticipationEventType,
    },
    node_manager::node::Node,
    Client,
};

use crate::account::AccountHandle;

impl AccountHandle {
    /// Stores participation information locally and returns the event.
    ///
    /// This will NOT store the node url and auth inside the client options.
    pub async fn register_participation_event(
        &self,
        id: ParticipationEventId,
        nodes: Vec<Node>,
    ) -> crate::Result<(ParticipationEvent, Vec<Node>)> {
        let mut client_builder = Client::builder().with_ignore_node_health();
        for node in &nodes {
            client_builder = client_builder.with_node_auth(node.url.as_str(), node.auth.clone())?;
        }
        let client = client_builder.finish()?;

        let event_data = client.event(&id).await?;

        let event = ParticipationEvent { id, data: event_data };

        self.storage_manager
            .lock()
            .await
            .insert_participation_event(self.read().await.index, id, event.clone(), nodes.clone())
            .await?;

        Ok((event, nodes))
    }

    /// Removes a previously registered participation event from local storage.
    pub async fn deregister_participation_event(&self, id: &ParticipationEventId) -> crate::Result<()> {
        self.storage_manager
            .lock()
            .await
            .remove_participation_event(self.read().await.index, id)
            .await?;
        Ok(())
    }

    /// Retrieves corresponding information for a participation event from local storage.
    pub async fn get_participation_event(
        &self,
        id: ParticipationEventId,
    ) -> crate::Result<Option<(ParticipationEvent, Vec<Node>)>> {
        Ok(self
            .storage_manager
            .lock()
            .await
            .get_participation_events(self.read().await.index)
            .await?
            .get(&id)
            .cloned())
    }

    /// Retrieves information for all registered participation events.
    pub async fn get_participation_events(
        &self,
    ) -> crate::Result<HashMap<ParticipationEventId, (ParticipationEvent, Vec<Node>)>> {
        self.storage_manager
            .lock()
            .await
            .get_participation_events(self.read().await.index)
            .await
    }

    /// Retrieves IDs of all events tracked by the client options node.
    pub async fn get_participation_event_ids(
        &self,
        event_type: Option<ParticipationEventType>,
    ) -> crate::Result<Vec<ParticipationEventId>> {
        Ok(self.client.events(event_type).await?.event_ids)
    }

    /// Retrieves the latest status of a given participation event.
    pub async fn get_participation_event_status(
        &self,
        id: &ParticipationEventId,
    ) -> crate::Result<ParticipationEventStatus> {
        Ok(self.get_client_for_event(id).await?.event_status(id, None).await?)
    }
}
