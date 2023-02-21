use iota_client::{api_types::plugins::participation::types::ParticipationEventId, node_manager::node::Node};
use serde::{Deserialize, Serialize};

/// Options when registering participation events.
/// If `events_to_register` is an empty `Vec` or `None`,
/// then every event being tracked by the node will be registered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipationEventRegistrationOptions {
    pub node: Node,
    pub events_to_register: Option<Vec<ParticipationEventId>>,
    pub events_to_ignore: Option<Vec<ParticipationEventId>>,
}
