// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    block::{
        output::{
            feature::{MetadataFeature, TagFeature},
            unlock_condition::AddressUnlockCondition,
            BasicOutputBuilder, Feature, Output, UnlockCondition,
        },
        payload::TaggedDataPayload,
    },
    node_api::participation::types::{participation::Participations, PARTICIPATION_TAG},
};

use crate::{
    account::{handle::AccountHandle, types::Transaction, TransactionOptions},
    Result,
};

impl AccountHandle {
    /// Returns an account's total voting power (voting or NOT voting).
    pub async fn get_voting_power(&self) -> Result<u64> {
        let voting_output = self.get_voting_output().await?;
        Ok(voting_output.output.amount())
    }

    /// Designates a given amount of tokens towards an account's "voting power" by creating a
    /// special output, which is really a basic one with some metadata.
    ///
    /// If not enough funds, throw error
    /// If voting use voting output (should only ever have one unless more spaces for more votes is needed)
    /// Remove metadata for any events that have expired (use event IDs to get cached event information, check event
    /// milestones in there against latest network milestone) Prioritize consuming outputs that are designated for
    /// voting but don't have any metadata (only possible if user increases voting power then increases again
    /// immediately after)
    pub async fn increase_voting_power(&self, amount: u64) -> Result<Transaction> {
        let token_supply = self.client().get_token_supply().await?;

        let (new_voting_output, tx_options) = match self.get_voting_output().await {
            Ok(previous_voting_output_data) => {
                let basic_output = if let Output::Basic(basic_output) = previous_voting_output_data.output {
                    basic_output
                } else {
                    unreachable!("voting output needs to be a basic output")
                };

                let mut updated_output_builder =
                    BasicOutputBuilder::from(&basic_output).with_amount(basic_output.amount() + amount)?;

                let mut participation_data = basic_output.features().metadata().map(|m| m.data()).unwrap_or(&[]);

                let participation_data =
                    if let Ok(mut participations) = Participations::from_bytes(&mut participation_data) {
                        // Remove ended participations
                        self.remove_ended_participation_events(&mut participations).await?;

                        let participation_bytes = participations.to_bytes()?;

                        updated_output_builder = updated_output_builder
                            .replace_feature(Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?))?;

                        participation_bytes
                    } else {
                        vec![]
                    };

                (
                    updated_output_builder.finish_output(token_supply)?,
                    Some(TransactionOptions {
                        // Use the previous voting output and additionally other for the additional amount
                        mandatory_inputs: Some(vec![previous_voting_output_data.output_id]),
                        tagged_data_payload: Some(TaggedDataPayload::new(
                            PARTICIPATION_TAG.as_bytes().to_vec(),
                            participation_data.to_vec(),
                        )?),
                        ..Default::default()
                    }),
                )
            }
            Err(_) => (
                BasicOutputBuilder::new_with_amount(amount)?
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                        self.public_addresses()
                            .await
                            .first()
                            .expect("account needs to have a public address")
                            .address
                            .inner,
                    )))
                    .add_feature(Feature::Tag(TagFeature::new(PARTICIPATION_TAG.as_bytes().to_vec())?))
                    .finish_output(token_supply)?,
                None,
            ),
        };

        self.send(vec![new_voting_output], tx_options).await
    }

    /// Reduces an account's "voting power" by a given amount.
    ///
    /// If amount is higher than actual voting power, throw error
    /// If voting and amount is equal to voting power, remove tagged data payload and output metadata
    /// Remove metadata for any events that have expired (use event IDs to get cached event information, check event
    /// milestones in there against latest network milestone) If an output is designated for voting but doesn't
    /// contain Prioritize consuming outputs that are designated for voting but don't have any metadata (only
    /// possible if user increases voting power then decreases immediately after)
    pub async fn decrease_voting_power(&self, amount: u64) -> Result<Transaction> {
        let token_supply = self.client().get_token_supply().await?;

        let previous_voting_output_data = self.get_voting_output().await?;

        let basic_output = if let Output::Basic(basic_output) = previous_voting_output_data.output {
            basic_output
        } else {
            unreachable!("voting output needs to be a basic output")
        };

        // If the amount to decrease is the amount of the output, then we just remove the features
        let (new_output, tagged_data_payload) = if amount == basic_output.amount() {
            (
                vec![BasicOutputBuilder::from(&basic_output)
                    .with_features([])
                    .finish_output(token_supply)?],
                None,
            )
        } else {
            let mut participation_data = basic_output.features().metadata().map(|m| m.data()).unwrap_or(&[]);

            let mut updated_output_builder =
                BasicOutputBuilder::from(&basic_output).with_amount(basic_output.amount() - amount)?;

            let participation_data = if let Ok(mut participations) = Participations::from_bytes(&mut participation_data)
            {
                // Remove ended participations
                self.remove_ended_participation_events(&mut participations).await?;

                let participation_bytes = participations.to_bytes()?;

                updated_output_builder = updated_output_builder
                    .replace_feature(Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?))?;

                participation_bytes
            } else {
                vec![]
            };

            (
                vec![updated_output_builder.finish_output(token_supply)?],
                Some(TaggedDataPayload::new(
                    PARTICIPATION_TAG.as_bytes().to_vec(),
                    participation_data.to_vec(),
                )?),
            )
        };

        self.send(
            new_output,
            Some(TransactionOptions {
                // Use the previous voting output and additionally others for possible additional required amount for
                // the remainder to reach the minimum required storage deposit
                mandatory_inputs: Some(vec![previous_voting_output_data.output_id]),
                tagged_data_payload,
                ..Default::default()
            }),
        )
        .await
    }
}
