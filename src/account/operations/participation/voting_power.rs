// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_client::{
    block::{
        output::{
            feature::{MetadataFeature, TagFeature},
            unlock_condition::AddressUnlockCondition,
            BasicOutput, BasicOutputBuilder, Feature, Output, UnlockCondition,
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
        Ok(self
            .get_voting_output()
            .await?
            // If no voting output exists, return 0
            .map_or(0, |v| v.output.amount()))
    }

    /// Designates a given amount of tokens towards an account's "voting power" by creating a
    /// special output, which is really a basic one with some metadata.
    ///
    /// If not enough funds, throws an error.
    /// If voting, use voting output (should only ever have one unless more space for more votes is needed).
    /// Removes metadata for any events that have expired (uses event IDs to get cached event information, checks event
    /// milestones in there against latest network milestone).
    /// Prioritizes consuming outputs that are designated for voting but don't have any metadata (only possible if user
    /// increases voting power then increases again immediately after).
    pub async fn increase_voting_power(&self, amount: u64) -> Result<Transaction> {
        let token_supply = self.client().get_token_supply().await?;

        let (new_output, tx_options) = match self.get_voting_output().await? {
            Some(current_output_data) => {
                let output = current_output_data.output.as_basic();

                // TODO checked addition

                let (new_output, tagged_data_payload) = self
                    .new_voting_output_and_tagged_data(output, output.amount() + amount, token_supply)
                    .await?;

                (
                    new_output,
                    Some(TransactionOptions {
                        // Use the previous voting output and additionally other for the additional amount.
                        mandatory_inputs: Some(vec![current_output_data.output_id]),
                        tagged_data_payload: Some(tagged_data_payload),
                        ..Default::default()
                    }),
                )
            }
            None => (
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

        self.send(vec![new_output], tx_options).await
    }

    /// Reduces an account's "voting power" by a given amount.
    ///
    /// If amount is higher than actual voting power, throws an error.
    /// If voting and amount is equal to voting power, removes tagged data payload and output metadata.
    /// Removes metadata for any events that have expired (uses event IDs to get cached event information, checks event
    /// milestones in there against latest network milestone).
    /// Prioritizes consuming outputs that are designated for voting but don't have any metadata (only possible if user
    /// increases voting power then decreases immediately after).
    pub async fn decrease_voting_power(&self, amount: u64) -> Result<Transaction> {
        let token_supply = self.client().get_token_supply().await?;
        let current_output_data = self
            .get_voting_output()
            .await?
            .ok_or_else(|| crate::Error::Voting("No unspent voting output found".to_string()))?;
        let output = current_output_data.output.as_basic();

        // TODO what is amount > output.amount() ?
        // If the amount to decrease is the amount of the output, then we just remove the features.
        let (new_output, tagged_data_payload) = if amount == output.amount() {
            (
                BasicOutputBuilder::from(output)
                    .with_features([])
                    .finish_output(token_supply)?,
                None,
            )
        } else {
            // TODO checked subtraction
            let (new_output, tagged_data_payload) = self
                .new_voting_output_and_tagged_data(output, output.amount() - amount, token_supply)
                .await?;

            (new_output, Some(tagged_data_payload))
        };

        self.send(
            vec![new_output],
            Some(TransactionOptions {
                // Use the previous voting output and additionally others for possible additional required amount for
                // the remainder to reach the minimum required storage deposit.
                mandatory_inputs: Some(vec![current_output_data.output_id]),
                tagged_data_payload,
                ..Default::default()
            }),
        )
        .await
    }

    async fn new_voting_output_and_tagged_data(
        &self,
        output: &BasicOutput,
        amount: u64,
        token_supply: u64,
    ) -> Result<(Output, TaggedDataPayload)> {
        let mut output_builder = BasicOutputBuilder::from(output).with_amount(amount)?;
        let mut participation_bytes = output.features().metadata().map(|m| m.data()).unwrap_or(&[]);

        let participation_bytes = if let Ok(mut participations) = Participations::from_bytes(&mut participation_bytes) {
            // Remove ended participations.
            self.remove_ended_participation_events(&mut participations).await?;

            let participation_bytes = participations.to_bytes()?;

            output_builder = output_builder
                .replace_feature(Feature::Metadata(MetadataFeature::new(participation_bytes.clone())?))?;

            participation_bytes
        } else {
            // TODO participation bytes are incorrect, should we really just ignore?
            vec![]
        };

        Ok((
            output_builder.finish_output(token_supply)?,
            TaggedDataPayload::new(PARTICIPATION_TAG.as_bytes().to_vec(), participation_bytes.to_vec())?,
        ))
    }
}
