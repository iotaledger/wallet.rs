// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::api_types::plugins::participation::types::ParticipationEventId;

use super::manager::StorageManager;
use crate::{
    account::operations::participation::ParticipationEventWithNodes, storage::constants::PARTICIPATION_EVENTS,
};

impl StorageManager {
    pub(crate) async fn insert_participation_event(
        &mut self,
        account_index: u32,
        event_with_nodes: ParticipationEventWithNodes,
    ) -> crate::Result<()> {
        log::debug!("insert_participation_event {}", event_with_nodes.id);

        let mut events = self
            .storage
            .get::<HashMap<ParticipationEventId, ParticipationEventWithNodes>>(&format!(
                "{PARTICIPATION_EVENTS}{account_index}"
            ))
            .await?
            .unwrap_or_default();

        events.insert(event_with_nodes.id, event_with_nodes);

        self.storage
            .set(&format!("{PARTICIPATION_EVENTS}{account_index}"), &events)
            .await?;

        Ok(())
    }

    pub(crate) async fn remove_participation_event(
        &mut self,
        account_index: u32,
        id: &ParticipationEventId,
    ) -> crate::Result<()> {
        log::debug!("remove_participation_event {id}");

        let mut events = match self
            .storage
            .get::<HashMap<ParticipationEventId, ParticipationEventWithNodes>>(&format!(
                "{PARTICIPATION_EVENTS}{account_index}"
            ))
            .await?
        {
            Some(events) => events,
            None => return Ok(()),
        };

        events.remove(id);

        self.storage
            .set(&format!("{PARTICIPATION_EVENTS}{account_index}"), &events)
            .await?;

        Ok(())
    }

    pub(crate) async fn get_participation_events(
        &self,
        account_index: u32,
    ) -> crate::Result<HashMap<ParticipationEventId, ParticipationEventWithNodes>> {
        log::debug!("get_participation_events");

        Ok(self
            .storage
            .get(&format!("{PARTICIPATION_EVENTS}{account_index}"))
            .await?
            .unwrap_or_default())
    }
}
