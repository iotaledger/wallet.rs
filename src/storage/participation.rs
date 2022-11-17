// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::{
    node_api::participation::types::{Event, EventId},
    node_manager::node::Node,
};

use super::manager::StorageManager;
use crate::storage::constants::PARTICIPATION_EVENTS;

impl StorageManager {
    pub(crate) async fn insert_participation_event(
        &mut self,
        id: EventId,
        event: Event,
        nodes: Vec<Node>,
    ) -> crate::Result<()> {
        log::debug!("insert_participation_event {id}");

        let mut events: HashMap<EventId, (Event, Vec<Node>)> = match self.storage.get(PARTICIPATION_EVENTS).await {
            Ok(events) => serde_json::from_str(&events)?,
            Err(crate::Error::RecordNotFound(_)) => HashMap::new(),
            Err(err) => return Err(err),
        };

        events.insert(id, (event, nodes));

        self.storage.set(PARTICIPATION_EVENTS, &events).await?;

        Ok(())
    }

    pub(crate) async fn remove_participation_event(&mut self, id: EventId) -> crate::Result<()> {
        log::debug!("remove_participation_event {id}");

        let mut events: HashMap<EventId, (Event, Vec<Node>)> =
            serde_json::from_str(&self.storage.get(PARTICIPATION_EVENTS).await?)?;

        events.remove(&id);

        self.storage.set(PARTICIPATION_EVENTS, &events).await?;

        Ok(())
    }

    pub(crate) async fn get_participation_events(&self) -> crate::Result<HashMap<EventId, (Event, Vec<Node>)>> {
        log::debug!("get_participation_events");

        Ok(serde_json::from_str(&self.storage.get(PARTICIPATION_EVENTS).await?)?)
    }
}
