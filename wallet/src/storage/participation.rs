// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use iota_client::{
    api_types::plugins::participation::{responses::OutputStatusResponse, types::ParticipationEventId},
    block::output::OutputId,
};

use super::manager::StorageManager;
use crate::{
    account::operations::participation::ParticipationEventWithNodes,
    storage::constants::{PARTICIPATION_CACHED_OUTPUTS, PARTICIPATION_EVENTS},
};

impl StorageManager {
    pub(crate) async fn insert_participation_event(
        &mut self,
        account_index: u32,
        event_with_nodes: ParticipationEventWithNodes,
    ) -> crate::Result<()> {
        log::debug!("insert_participation_event {}", event_with_nodes.id);

        let mut events: HashMap<ParticipationEventId, ParticipationEventWithNodes> = match self
            .storage
            .get(&format!("{PARTICIPATION_EVENTS}{account_index}"))
            .await?
        {
            Some(events) => serde_json::from_str(&events)?,
            None => HashMap::new(),
        };

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

        let mut events: HashMap<ParticipationEventId, ParticipationEventWithNodes> = match self
            .storage
            .get(&format!("{PARTICIPATION_EVENTS}{account_index}"))
            .await?
        {
            Some(events) => serde_json::from_str(&events)?,
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

        match self
            .storage
            .get(&format!("{PARTICIPATION_EVENTS}{account_index}"))
            .await?
        {
            Some(events) => Ok(serde_json::from_str(&events)?),
            None => Ok(HashMap::new()),
        }
    }

    pub(crate) async fn set_cached_participation_output_status(
        &mut self,
        account_index: u32,
        outputs_participation: HashMap<OutputId, OutputStatusResponse>,
    ) -> crate::Result<()> {
        log::debug!("set_cached_participation");

        self.storage
            .set(
                &format!("{PARTICIPATION_CACHED_OUTPUTS}{account_index}"),
                outputs_participation,
            )
            .await?;
        Ok(())
    }

    pub(crate) async fn get_cached_participation_output_status(
        &self,
        account_index: u32,
    ) -> crate::Result<HashMap<OutputId, OutputStatusResponse>> {
        log::debug!("get_cached_participation");

        match self
            .storage
            .get(&format!("{PARTICIPATION_CACHED_OUTPUTS}{account_index}"))
            .await?
        {
            Some(cached_outputs) => Ok(serde_json::from_str(&cached_outputs)?),
            None => Ok(HashMap::new()),
        }
    }
}
