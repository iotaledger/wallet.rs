// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    api_types::plugins::participation::types::{
        Participation, ParticipationEventId, Participations, PARTICIPATION_TAG,
    },
    block::{
        output::{
            feature::{MetadataFeature, TagFeature},
            BasicOutputBuilder, Feature,
        },
        payload::TaggedDataPayload,
    },
};

use crate::{
    account::{types::Transaction, AccountHandle, TransactionOptions},
    Result,
};

impl AccountHandle {
    /// Casts a given number of votes for a given (voting) event.
    ///
    /// If voting for other events, continues voting for them.
    /// Removes metadata for any event that has expired (uses event IDs to get cached event information, checks event
    /// milestones in there against latest network milestone).
    /// If already voting for this event, overwrites existing output metadata.
    /// If existing voting output(s) do NOT have enough funds (or don't exist), throws an error.
    /// If exceeds output metadata limit, throws an error (although better if automatically handled, but has UX
    /// implications).
    /// If event has expired, throws an error (do NOT remove previous votes).
    ///
    /// This is an add OR update function, not just add.
    /// This should use regular client options, NOT specific node for the event.
    pub async fn vote(&self, event_id: Option<ParticipationEventId>, answers: Option<Vec<u8>>) -> Result<Transaction> {
        if let Some(event_id) = event_id {
            let event_status = self.get_participation_event_status(&event_id).await?;

            // Checks if voting event is still running.
            if event_status.status() == "ended" {
                return Err(crate::Error::Voting(format!("event {event_id} already ended")));
            }
        }

        // TODO check if answers match the questions ?

        let voting_output = self
            .get_voting_output()
            .await?
            .ok_or_else(|| crate::Error::Voting("No unspent voting output found".to_string()))?;
        let output = voting_output.output.as_basic();

        // Updates or creates participation.
        let participation_bytes = match output.features().metadata() {
            Some(metadata) => {
                let mut participations = Participations::from_bytes(&mut metadata.data())?;

                // Removes ended participations.
                self.remove_ended_participation_events(&mut participations).await?;

                if let Some(event_id) = event_id {
                    participations.add_or_replace(Participation {
                        event_id,
                        answers: answers.unwrap_or_default(),
                    });
                }

                participations
            }
            None => {
                if let Some(event_id) = event_id {
                    Participations {
                        participations: vec![Participation {
                            event_id,
                            answers: answers.unwrap_or_default(),
                        }],
                    }
                } else {
                    return Err(crate::Error::Voting("No event to vote for".to_string()));
                }
            }
        }
        .to_bytes()?;

        let new_output = BasicOutputBuilder::from(output)
            // TODO maybe replace ?
            .with_features(vec![
                Feature::Tag(TagFeature::new(PARTICIPATION_TAG.as_bytes().to_vec())?),
                Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?),
            ])
            .finish_output(self.client().get_token_supply().await?)?;

        self.send(
            vec![new_output],
            Some(TransactionOptions {
                // Only use previous voting output as input.
                custom_inputs: Some(vec![voting_output.output_id]),
                mandatory_inputs: Some(vec![voting_output.output_id]),
                tagged_data_payload: Some(TaggedDataPayload::new(
                    PARTICIPATION_TAG.as_bytes().to_vec(),
                    participation_bytes,
                )?),
                ..Default::default()
            }),
        )
        .await
    }

    /// Removes metadata corresponding to a given (voting) event ID from any outputs that contains it.
    ///
    /// If voting for other events, continues voting for them.
    /// Removes metadata for any event that has expired (use event IDs to get cached event information, checks event
    /// milestones in there against latest network milestone).
    /// TODO: is it really doing that ?
    /// If multiple outputs contain metadata for this event, removes all of them.
    /// If NOT already voting for this event, throws an error (e.g. output with this event ID not found).
    pub async fn stop_participating(&self, event_id: ParticipationEventId) -> Result<Transaction> {
        let voting_output = self
            .get_voting_output()
            .await?
            .ok_or_else(|| crate::Error::Voting("No unspent voting output found".to_string()))?;
        let output = voting_output.output.as_basic();

        // Removes participation.
        let participation_bytes = match output.features().metadata() {
            Some(metadata) => {
                let mut participations = Participations::from_bytes(&mut metadata.data())?;

                let length_before = participations.participations.len();

                // TODO use remove return when merged
                participations.remove(&event_id);

                if length_before == participations.participations.len() {
                    // TODO should this really be an error ?
                    return Err(crate::Error::Voting(format!(
                        "currently not participating for {event_id}"
                    )));
                }

                // Removes ended participations.
                self.remove_ended_participation_events(&mut participations).await?;

                participations
            }
            None => {
                // TODO should this really be an error ?
                return Err(crate::Error::Voting(format!(
                    "currently not participating for {event_id}"
                )));
            }
        }
        .to_bytes()?;

        let new_output = BasicOutputBuilder::from(output)
            // TODO maybe replace ?
            .with_features(vec![
                Feature::Tag(TagFeature::new(PARTICIPATION_TAG.as_bytes().to_vec())?),
                Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?),
            ])
            .finish_output(self.client().get_token_supply().await?)?;

        self.send(
            vec![new_output],
            Some(TransactionOptions {
                // Only use previous voting output as input.
                custom_inputs: Some(vec![voting_output.output_id]),
                mandatory_inputs: Some(vec![voting_output.output_id]),
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
