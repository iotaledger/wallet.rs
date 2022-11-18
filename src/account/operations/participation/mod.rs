// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// We are requiring the user to manually increase/decrease the “voting power”, which requires wallet.rs to
// designate some amount of funds as unspendable. They become spendable again when the user reduces the “voting
// power”.
// This is done by creating a special “voting output” that adheres to the following rules; NOT by sending to a differing
// address.
// If the user has designated funds to vote with, the resulting output MUST NOT be used for input selection.

pub mod voting;
pub mod voting_power;

use std::collections::HashMap;

use iota_client::{
    block::output::{Output, OutputId},
    node_api::participation::{
        responses::TrackedParticipation,
        types::{participation::Participations, EventId, EventStatus, PARTICIPATION_TAG},
    },
    Client,
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{handle::AccountHandle, OutputData},
    Result,
};

/// An object containing an account's entire participation overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountParticipationOverview {
    /// Output participations for events
    pub participations: HashMap<EventId, HashMap<OutputId, TrackedParticipation>>,
}

impl AccountHandle {
    /// Calculates a voting overview for an account.
    pub async fn get_participation_overview(&self) -> Result<AccountParticipationOverview> {
        // could use the address endpoint in the future when https://github.com/iotaledger/inx-participation/issues/50 is done

        let outputs = self.outputs(None).await?;
        let participation_outputs: Vec<OutputData> = outputs
            .into_iter()
            .filter(|o| {
                // only basic outputs can be participation outputs
                if let Output::Basic(basic_output) = &o.output {
                    // output needs to have the participation tag and a metadata feature
                    if let Some(tag_feature) = basic_output.features().tag() {
                        tag_feature.tag() == PARTICIPATION_TAG.as_bytes()
                            && basic_output.features().metadata().is_some()
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect();

        let mut participations: HashMap<EventId, HashMap<OutputId, TrackedParticipation>> = HashMap::new();

        for output_data in participation_outputs {
            let metadata = match output_data.output.features().map(|f| f.metadata()) {
                Some(Some(metadata)) => metadata,
                // no participation in this output, skip
                _ => continue,
            };
            let event_ids = if let Ok(participations) = Participations::from_bytes(&mut metadata.data()) {
                participations
                    .participations
                    .into_iter()
                    .map(|p| p.event_id)
                    .collect::<Vec<EventId>>()
            } else {
                // no valid participation in this output, skip
                continue;
            };

            // TODO: which client to use if there are multiple events and one node isn't tracking all of them?
            let event_client = if let Some(event_id) = event_ids.first() {
                self.get_client_for_event(event_id).await?
            } else {
                self.client().clone()
            };

            if let Ok(response) = event_client.output_status(&output_data.output_id).await {
                for (event_id, participation) in response.participations {
                    participations
                        .entry(event_id)
                        .and_modify(|output_participations| {
                            output_participations.insert(output_data.output_id, participation.clone());
                        })
                        .or_insert_with(|| HashMap::from([(output_data.output_id, participation)]));
                }
            }
        }

        Ok(AccountParticipationOverview { participations })
    }

    /// Returns the voting output ("PARTICIPATION" tag).
    ///
    /// If multiple outputs with this tag exist, the one with the largest amount will be returned.
    pub async fn get_voting_output(&self) -> Result<OutputData> {
        let mut participation_outputs = self
            .unspent_outputs(None)
            .await?
            .into_iter()
            .filter(|output_data| {
                if let Output::Basic(basic_output) = &output_data.output {
                    if let Some(tag) = basic_output.features().tag() {
                        tag.tag() == PARTICIPATION_TAG.as_bytes()
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect::<Vec<OutputData>>();

        // Sort by amount
        participation_outputs.sort_by(|a, b| a.output.amount().cmp(&b.output.amount()));

        participation_outputs
            // Use output with largest amount
            .last()
            .cloned()
            .ok_or_else(|| crate::Error::VotingError("No unspent voting output found".to_string()))
    }

    /// Get client for an event, if event isn't found, the client from the account will be returned.
    pub(crate) async fn get_client_for_event(&self, id: &EventId) -> crate::Result<Client> {
        let events = match self.storage_manager.lock().await.get_participation_events().await {
            Ok(events) => events,
            _ => return Ok(self.client().clone()),
        };

        let event = match events.get(id) {
            Some(event) => event,
            None => return Ok(self.client().clone()),
        };

        let mut client_builder = Client::builder().with_ignore_node_health();
        for node in &event.1 {
            client_builder = client_builder.with_node_auth(node.url.as_str(), node.auth.clone())?;
        }
        let client = client_builder.finish()?;

        Ok(client)
    }

    /// Check if events in the participations ended and remove them.
    pub(crate) async fn remove_ended_participation_events(
        &self,
        mut participations: Participations,
    ) -> crate::Result<Participations> {
        let latest_milestone_index = self.client().get_info().await?.node_info.status.latest_milestone.index;

        // TODO: don't return error here if there is nothing?
        let events = self.storage_manager.lock().await.get_participation_events().await?;

        for participation in participations.participations.clone().iter() {
            if let Some((event, _nodes)) = events.get(&participation.event_id) {
                if event.data.milestone_index_end() < &latest_milestone_index {
                    participations.remove(&participation.event_id);
                }
            } else {
                // if not found in local events, try to get the event status from the client
                if let Ok(event_status) = self.get_participation_event_status(&participation.event_id).await {
                    if event_status.status() == "ended" {
                        participations.remove(&participation.event_id);
                    }
                }
            }
        }
        Ok(participations)
    }

    /// Retrieves the latest status of a given participation event.
    pub(crate) async fn get_participation_event_status(&self, id: &EventId) -> crate::Result<EventStatus> {
        let client = self.get_client_for_event(id).await?;

        let events_status = client.event_status(id, None).await?;

        Ok(events_status)
    }
}
