// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    block::{
        output::{
            feature::{MetadataFeature, TagFeature},
            BasicOutputBuilder, Feature, Output,
        },
        payload::TaggedDataPayload,
    },
    node_api::participation::types::{
        participation::{Participation, Participations},
        EventId, PARTICIPATION_TAG,
    },
};

use crate::{
    account::{types::Transaction, AccountHandle, TransactionOptions},
    Result,
};

impl AccountHandle {
    /// Casts a given number of votes for a given (voting) event.
    ///
    /// If voting for other events, continue voting for other events
    /// Remove metadata for any events that have expired (use event IDs to get cached event information, check event
    /// milestones in there against latest network milestone) If already voting for this event, overwrite existing
    /// output metadata. If existing voting output(s) do NOT have enough funds (or don't exist), throw error
    /// If exceeds output metadata limit, throw error (although better if automatically handled... but has UX
    /// implications...) If event has expired, throw error (do NOT remove previous votes)
    ///
    /// This is an add OR update function; not just add
    /// This should use regular client options; NOT specific node for the event
    pub async fn vote(&self, event_id: EventId, answers: Vec<u8>) -> Result<Transaction> {
        // Check if voting event is still running
        let event_status = self.get_participation_event_status(&event_id).await?;
        if event_status.status() == "ended" {
            return Err(crate::Error::Voting(format!("event {event_id} already ended")));
        }

        let voting_output = self.get_voting_output().await?;

        let basic_output = if let Output::Basic(basic_output) = voting_output.output {
            basic_output
        } else {
            unreachable!("voting output must be a basic output")
        };

        let metadata = basic_output.features().metadata();

        // Update or create participation
        let participation_bytes = match metadata {
            Some(metadata) => {
                let mut slice: &[u8] = metadata.data();
                let mut participations = Participations::from_bytes(&mut slice)?;

                // Remove ended participations
                self.remove_ended_participation_events(&mut participations).await?;

                participations.add_or_replace(Participation { event_id, answers });

                participations
            }
            None => Participations {
                participations: vec![Participation { event_id, answers }],
            },
        }
        .to_bytes()?;

        let token_supply = self.client().get_token_supply().await?;

        let updated_output = BasicOutputBuilder::from(&basic_output)
            .with_features(vec![
                Feature::Tag(TagFeature::new(PARTICIPATION_TAG.as_bytes().to_vec())?),
                Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?),
            ])
            .finish_output(token_supply)?;

        self.send(
            vec![updated_output],
            Some(TransactionOptions {
                // only use previous voting output as input
                custom_inputs: Some(vec![voting_output.output_id]),
                tagged_data_payload: Some(TaggedDataPayload::new(
                    PARTICIPATION_TAG.as_bytes().to_vec(),
                    participation_bytes,
                )?),
                ..Default::default()
            }),
        )
        .await
    }

    /// Removes metadata corresponding to a given (voting) event ID from any outputs that contain it.
    ///
    /// If voting for other events, continue voting for other events
    /// Remove metadata for any events that have expired (use event IDs to get cached event information, check event
    /// milestones in there against latest network milestone) If multiple outputs contain metadata for this event,
    /// remove all of them. If NOT already voting for event, throw error (e.g. output with this event ID not found)
    pub async fn stop_participating(&self, event_id: EventId) -> Result<Transaction> {
        let voting_output = self.get_voting_output().await?;

        let basic_output = if let Output::Basic(basic_output) = voting_output.output {
            basic_output
        } else {
            unreachable!("voting output needs to be a basic output")
        };

        let metadata = basic_output.features().metadata();

        // Remove participation
        let participation_bytes = match metadata {
            Some(metadata) => {
                let mut slice: &[u8] = metadata.data();
                let mut participations = Participations::from_bytes(&mut slice)?;

                let length_before = participations.participations.len();

                participations.remove(&event_id);

                if length_before == participations.participations.len() {
                    return Err(crate::Error::Voting(format!(
                        "currently not participating for {event_id}"
                    )));
                }

                // Remove ended participations
                self.remove_ended_participation_events(&mut participations).await?;

                participations
            }
            None => {
                return Err(crate::Error::Voting(format!(
                    "currently not participating for {event_id}"
                )));
            }
        }
        .to_bytes()?;

        let token_supply = self.client().get_token_supply().await?;

        let updated_output = BasicOutputBuilder::from(&basic_output)
            .with_features(vec![
                Feature::Tag(TagFeature::new(PARTICIPATION_TAG.as_bytes().to_vec())?),
                Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?),
            ])
            .finish_output(token_supply)?;

        self.send(
            vec![updated_output],
            Some(TransactionOptions {
                // only use previous voting output as input
                custom_inputs: Some(vec![voting_output.output_id]),
                tagged_data_payload: Some(TaggedDataPayload::new(
                    PARTICIPATION_TAG.as_bytes().to_vec(),
                    participation_bytes,
                )?),
                ..Default::default()
            }),
        )
        .await
    }
}
