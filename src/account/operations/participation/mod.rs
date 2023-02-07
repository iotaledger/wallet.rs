// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// We are requiring the user to manually increase/decrease the “voting power”, which requires the wallet to designate
// some amount of funds as unspendable.
// They become spendable again when the user reduces the “voting power”.
// This is done by creating a special “voting output” that adheres to the following rules, NOT by sending to a different
// address.
// If the user has designated funds to vote with, the resulting output MUST NOT be used for input selection.

pub mod event;
pub mod voting;
pub mod voting_power;

use std::collections::{hash_map::Entry, HashMap};

use iota_client::{
    api_types::plugins::participation::{
        responses::TrackedParticipation,
        types::{ParticipationEventData, ParticipationEventId, Participations, PARTICIPATION_TAG},
    },
    block::output::{unlock_condition::UnlockCondition, Output, OutputId},
    node_manager::node::Node,
    Client,
};
use serde::{Deserialize, Serialize};

use crate::{
    account::{handle::AccountHandle, OutputData},
    task, Result,
};

/// An object containing an account's entire participation overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountParticipationOverview {
    /// Output participations for events.
    pub participations: HashMap<ParticipationEventId, HashMap<OutputId, TrackedParticipation>>,
}

/// A participation event with the provided client nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationEventWithNodes {
    /// The event id.
    pub id: ParticipationEventId,
    /// Information about a voting or staking event.
    pub data: ParticipationEventData,
    /// Provided client nodes for this event.
    pub nodes: Vec<Node>,
}

impl AccountHandle {
    /// Calculates the voting overview of an account.
    pub async fn get_participation_overview(&self) -> Result<AccountParticipationOverview> {
        // TODO: Could use the address endpoint in the future when https://github.com/iotaledger/inx-participation/issues/50 is done.

        let outputs = self.outputs(None).await?;
        let participation_outputs = outputs
            .into_iter()
            .filter(|output_data| {
                is_valid_participation_output(&output_data.output)
                // Check that the metadata exists, because otherwise we aren't participating for anything
                    && output_data.output.features().and_then(|f| f.metadata()).is_some()
            })
            .collect::<Vec<OutputData>>();

        let mut participations: HashMap<ParticipationEventId, HashMap<OutputId, TrackedParticipation>> = HashMap::new();

        for output_data_chunk in participation_outputs.chunks(100).map(|x| x.to_vec()) {
            let mut tasks = Vec::new();
            for output_data in output_data_chunk {
                // PANIC: the filter already checks that the metadata exists.
                let metadata = output_data.output.features().and_then(|f| f.metadata()).unwrap();
                // TODO don't really need to collect if we only need the first
                let event_ids = if let Ok(participations) = Participations::from_bytes(&mut metadata.data()) {
                    participations
                        .participations
                        .into_iter()
                        .map(|p| p.event_id)
                        .collect::<Vec<ParticipationEventId>>()
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

                tasks.push(async move {
                    task::spawn(async move { (event_client.output_status(&output_data.output_id).await, output_data) })
                        .await
                });
            }

            let results = futures::future::try_join_all(tasks).await?;
            for (result, output_data) in results {
                match result {
                    Ok(status) => {
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
                    Err(iota_client::Error::NotFound(_)) => {}
                    Err(e) => return Err(crate::Error::Client(e.into())),
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
            .filter(|output_data| is_valid_participation_output(&output_data.output))
            .max_by_key(|output_data| output_data.output.amount())
            .cloned())
    }

    /// Gets client for an event.
    /// If event isn't found, the client from the account will be returned.
    pub(crate) async fn get_client_for_event(&self, id: &ParticipationEventId) -> crate::Result<Client> {
        let events = self
            .storage_manager
            .lock()
            .await
            .get_participation_events(self.read().await.index)
            .await?;

        let event_with_nodes = match events.get(id) {
            Some(event_with_nodes) => event_with_nodes,
            None => return Ok(self.client().clone()),
        };

        let mut client_builder = Client::builder().with_ignore_node_health();
        for node in &event_with_nodes.nodes {
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

        let events = self
            .storage_manager
            .lock()
            .await
            .get_participation_events(self.read().await.index)
            .await?;

        // TODO try to remove this clone
        for participation in participations.participations.clone().iter() {
            if let Some(event_with_nodes) = events.get(&participation.event_id) {
                if event_with_nodes.data.milestone_index_end() < &latest_milestone_index {
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
}

fn is_valid_participation_output(output: &Output) -> bool {
    // Only basic outputs can be participation outputs.
    if let Output::Basic(basic_output) = &output {
        // Valid participation outputs can only have the AddressUnlockCondition.
        let [UnlockCondition::Address(_)] = basic_output.unlock_conditions().as_ref() else {
            return false;
        };
        if let Some(tag) = basic_output.features().tag() {
            tag.tag() == PARTICIPATION_TAG.as_bytes()
        } else {
            false
        }
    } else {
        false
    }
}
