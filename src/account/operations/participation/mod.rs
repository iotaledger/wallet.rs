// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// We are requiring the user to manually increase/decrease the “voting power”, which requires the wallet to designate
// some amount of funds as unspendable.
// They become spendable again when the user reduces the “voting power”.
// This is done by creating a special “voting output” that adheres to the following rules, NOT by sending to a different
// address.
// If the user has designated funds to vote with, the resulting output MUST NOT be used for input selection.

pub mod voting;
pub mod voting_power;

use std::collections::{hash_map::Entry, HashMap};

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
    /// Output participations for events.
    participations: HashMap<EventId, HashMap<OutputId, TrackedParticipation>>,
}

impl AccountHandle {
    /// Calculates the voting overview of an account.
    pub async fn get_participation_overview(&self) -> Result<AccountParticipationOverview> {
        // TODO: Could use the address endpoint in the future when https://github.com/iotaledger/inx-participation/issues/50 is done.

        let outputs = self.outputs(None).await?;
        let participation_outputs = outputs.iter().filter(|output| {
            // Only basic outputs can be participation outputs.
            if let Output::Basic(basic_output) = &output.output {
                // Output needs to have the participation tag and a metadata feature.
                if let Some(tag_feature) = basic_output.features().tag() {
                    tag_feature.tag() == PARTICIPATION_TAG.as_bytes() && basic_output.features().metadata().is_some()
                } else {
                    false
                }
            } else {
                false
            }
        });

        let mut participations: HashMap<EventId, HashMap<OutputId, TrackedParticipation>> = HashMap::new();

        for output_data in participation_outputs {
            // PANIC: the filter already checks that the metadata exists.
            let metadata = output_data.output.features().and_then(|f| f.metadata()).unwrap();
            // TODO don't really need to collect if we only need the first
            let event_ids = if let Ok(participations) = Participations::from_bytes(&mut metadata.data()) {
                participations
                    .participations
                    .into_iter()
                    .map(|p| p.event_id)
                    .collect::<Vec<EventId>>()
            } else {
                // No valid participation in this output, skip it.
                continue;
            };

            // TODO: which client to use if there are multiple events and one node isn't tracking all of them?
            let event_client = if let Some(event_id) = event_ids.first() {
                self.get_client_for_event(event_id).await?
            } else {
                self.client().clone()
            };

            if let Ok(status) = event_client.output_status(&output_data.output_id).await {
                for (event_id, participation) in status.participations {
                    match participations.entry(event_id) {
                        Entry::Vacant(entry) => {
                            entry.insert(HashMap::from([(output_data.output_id, participation)]));
                        }
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().insert(output_data.output_id, participation);
                        }
                    }
                }
            }
        }

        Ok(AccountParticipationOverview { participations })
    }

    /// Returns the voting output ("PARTICIPATION" tag).
    ///
    /// If multiple outputs with this tag exist, the one with the largest amount will be returned.
    pub async fn get_voting_output(&self) -> Result<Option<OutputData>> {
        Ok(self
            .unspent_outputs(None)
            .await?
            .iter()
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
            .max_by_key(|output_data| output_data.output.amount())
            .cloned())
    }

    /// Gets client for an event.
    /// If event isn't found, the client from the account will be returned.
    pub(crate) async fn get_client_for_event(&self, id: &EventId) -> crate::Result<Client> {
        let events = self.storage_manager.lock().await.get_participation_events().await?;

        let event = match events.get(id) {
            Some(event) => event,
            None => return Ok(self.client().clone()),
        };

        let mut client_builder = Client::builder().with_ignore_node_health();
        for node in &event.1 {
            client_builder = client_builder.with_node_auth(node.url.as_str(), node.auth.clone())?;
        }

        Ok(client_builder.finish()?)
    }

    /// Checks if events in the participations ended and removes them.
    pub(crate) async fn remove_ended_participation_events(
        &self,
        participations: &mut Participations,
    ) -> crate::Result<()> {
        let latest_milestone_index = self.client().get_info().await?.node_info.status.latest_milestone.index;

        let events = self.storage_manager.lock().await.get_participation_events().await?;

        // TODO try to remove this clone
        for participation in participations.participations.clone().iter() {
            if let Some((event, _nodes)) = events.get(&participation.event_id) {
                if event.data.milestone_index_end() < &latest_milestone_index {
                    participations.remove(&participation.event_id);
                }
            } else {
                // If not found in local events, try to get the event status from the client.
                if let Ok(event_status) = self.get_participation_event_status(&participation.event_id).await {
                    if event_status.status() == "ended" {
                        participations.remove(&participation.event_id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Retrieves the latest status of a given participation event.
    pub(crate) async fn get_participation_event_status(&self, id: &EventId) -> crate::Result<EventStatus> {
        Ok(self.get_client_for_event(id).await?.event_status(id, None).await?)
    }
}
