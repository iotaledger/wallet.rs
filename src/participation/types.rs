// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::participation::response_types::{EventInformation, EventStatus};

use serde::{Deserialize, Serialize};

use std::{convert::TryInto, io::Read};

/// Particiapation index
pub(crate) const PARTICIPATE: &str = "PARTICIPATE";
/// Event id for the shimmer staking
pub(crate) const SHIMMER_EVENT_ID: &str = "6652309b69b5b93066a761ee17244b0bff49d365e1677b95ee4e4fc77ad8ddb8";
/// Event id for the assembly staking
pub(crate) const ASSEMBLY_EVENT_ID: &str = "b089e0141e800fbfcdad7effac311c03b958135f4b0a8fa708a552ea5aadec44";

/// Possible participation event types
pub enum ParticipationEventType {
    /// Voting event
    Voting,
    /// Staking event
    Staking,
}

/// All information about an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// The event id
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// Information about a voting or staking event
    pub information: EventInformation,
    /// Event status, with the information if it started and the total staked funds
    pub status: EventStatus,
}

/// Overview of the accounts with their participations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipatingAccounts {
    /// All accounts
    pub accounts: Vec<ParticipatingAccount>,
}

/// Overview of an accounts with its participations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipatingAccount {
    /// The index of the account
    #[serde(rename = "accountIndex")]
    pub account_index: usize,
    /// The events the acount participates at the moment
    pub participations: Vec<Participation>,
    /// Fund that are currently staking for assembly
    #[serde(rename = "assemblyStakedFunds")]
    pub assembly_staked_funds: u64,
    /// Generated assembly rewards below minimum required amount
    #[serde(rename = "assemblyRewardsBelowMinimum")]
    pub assembly_rewards_below_minimum: u64,
    /// Generated assembly rewards
    #[serde(rename = "assemblyRewards")]
    pub assembly_rewards: u64,
    /// Funds that aren't currently staking for assembly
    #[serde(rename = "assemblyUnstakedFunds")]
    pub assembly_unstaked_funds: u64,
    /// Fund that are currently staking for shimmer
    #[serde(rename = "shimmerStakedFunds")]
    pub shimmer_staked_funds: u64,
    /// Generated shimmer rewards below minimum required amount
    #[serde(rename = "shimmerRewardsBelowMinimum")]
    pub shimmer_rewards_below_minimum: u64,
    /// Generated shimmer rewards
    #[serde(rename = "shimmerRewards")]
    pub shimmer_rewards: u64,
    /// Funds that aren't currently staking for shimmer
    #[serde(rename = "shimmerUnstakedFunds")]
    pub shimmer_unstaked_funds: u64,
}

/// Participation information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participation {
    /// A staking or voting event id, hex encoded [u8; 32]
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// Answers for a voting event, can be empty
    pub answers: Vec<u8>,
}

/// Participation information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participations {
    /// Multiple participations that happen at the same time
    pub participations: Vec<Participation>,
}

impl Participations {
    // https://github.com/alexsporn/treasury/blob/main/specifications/chrysalis-referendum-rfc.md#structure-of-the-participation
    /// Serialize to bytes
    pub(crate) fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![self
            .participations
            .len()
            .try_into()
            .map_err(|_| crate::Error::InvalidParticipations)?];

        for participation in &self.participations {
            bytes.extend(hex::decode(&participation.event_id).map_err(|_| crate::Error::InvalidParticipations)?);
            bytes.push(
                participation
                    .answers
                    .len()
                    .try_into()
                    .map_err(|_| crate::Error::InvalidParticipations)?,
            );
            for answer in &participation.answers {
                bytes.push(*answer);
            }
        }
        Ok(bytes)
    }
    /// Deserialize from bytes
    pub(crate) fn from_bytes<R: Read + ?Sized>(bytes: &mut R) -> crate::Result<Participations> {
        let mut participations = Vec::new();
        let mut participations_len = [0u8; 1];
        bytes.read_exact(&mut participations_len)?;

        for _ in 0..participations_len[0] {
            let mut event_id: [u8; 32] = [0u8; 32];
            bytes.read_exact(&mut event_id)?;

            let mut answers_len = [0u8; 1];
            bytes.read_exact(&mut answers_len)?;

            let mut answers = Vec::new();
            for _ in 0..answers_len[0] {
                let mut answer = [0u8; 1];
                bytes.read_exact(&mut answer)?;
                answers.push(answer[0]);
            }

            participations.push(Participation {
                event_id: hex::encode(event_id),
                answers,
            });
        }

        Ok(Participations { participations })
    }
}

#[cfg(test)]
mod tests {
    use super::Participations;
    use crate::participation::types::Participation;

    #[test]
    fn serialize_deserialize() {
        let participations = Participations {
            participations: vec![
                Participation {
                    event_id: "09c2338f3acd51e626cc074d1abcb12d747076ddfccd5215d8f2f21af1aac111".to_string(),
                    answers: vec![0, 1],
                },
                Participation {
                    event_id: "0207c34ae298b90d85455eee718037ad84a46bd784cbe5fdd8c534cc955efa1f".to_string(),
                    answers: vec![],
                },
            ],
        };
        let participation_bytes = participations.to_bytes().unwrap();
        let mut slice: &[u8] = &participation_bytes;
        let deserialized_participations: Participations = Participations::from_bytes(&mut slice).unwrap();

        assert_eq!(participations, deserialized_participations);
    }
}

// events
// pub struct StakedAccount {
//     messages: Vec<Message>,
// }
