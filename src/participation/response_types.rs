// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// Response for get_events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventIds {
    #[serde(rename = "eventIds")]
    pub event_ids: Vec<String>,
}

/// Information about a voting or staking event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInformation {
    name: String,
    #[serde(rename = "milestoneIndexCommence")]
    milestone_index_commence: u32,
    #[serde(rename = "milestoneIndexStart")]
    milestone_index_start: u32,
    #[serde(rename = "milestoneIndexEnd")]
    milestone_index_end: u32,
    payload: EventPayload,
    #[serde(rename = "additionalInfo")]
    additional_info: String,
}

/// Event payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    /// Voting payload
    VotingEventPayload(VotingEventPayload),
    /// Staking payload
    StakingEventPayload(StakingEventPayload),
}

/// Payload for a staking event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingEventPayload {
    #[serde(rename = "type")]
    kind: u32,
    text: String,
    symbol: String,
    numerator: u64,
    denominator: u64,
    #[serde(rename = "requiredMinimumRewards")]
    required_minimum_rewards: u64,
    #[serde(rename = "additionalInfo")]
    additional_info: String,
}

/// Payload for a voting event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingEventPayload {
    #[serde(rename = "type")]
    kind: u32,
    questions: Vec<Questions>,
}

/// Question for a voting event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Questions {
    text: String,
    answers: Vec<Answer>,
    #[serde(rename = "additionalInfo")]
    additional_info: String,
}

/// Answer in a voting event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    value: u8,
    text: String,
    #[serde(rename = "additionalInfo")]
    additional_info: String,
}

/// Event status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStatus {
    #[serde(rename = "milestoneIndex")]
    milestone_index: u32,
    status: String,
    questions: Option<Vec<Answers>>,
    checksum: String,
}

/// Answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answers {
    answers: Vec<AnswerStatus>,
}

/// Answer status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerStatus {
    value: u8,
    current: u64,
    accumulated: u64,
}

/// Staking rewards for an address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressStakingStatus {
    /// Rewards for staking events
    pub rewards: HashMap<String, StakingStatus>,
}

/// Staking rewards for an address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStatus {
    /// Staked amount
    pub amount: u64,
    /// Currency symbol
    pub symbol: String,
    /// If the required minimum staking reward is reached
    #[serde(rename = "minimumReached")]
    pub minimum_reached: bool,
}

/// All participations for an output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStatusResponse {
    /// Participations an output takes part of
    pub participations: HashMap<String, TrackedParticipation>,
}

/// The participation information for an output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedParticipation {
    /// Message id that created the output
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// Amount of the output that participates
    pub amount: u64,
    /// The milestone at which the output started to stake funds
    #[serde(rename = "startMilestoneIndex")]
    pub start_milestone_index: u32,
    /// The milestone at which the output stopped to stake funds
    #[serde(rename = "endMilestoneIndex")]
    pub end_milestone_index: u32,
}
