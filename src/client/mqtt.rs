// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// /// The MQTT broker options.
// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct BrokerOptions {
//     // We need to use `pub` here or these is no way to let the user create BrokerOptions
//     #[serde(rename = "automaticDisconnect")]
//     /// Whether the MQTT broker should be automatically disconnected when all topics are unsubscribed or not.
//     pub automatic_disconnect: Option<bool>,
//     /// timeout of the mqtt broker.
//     pub timeout: Option<Duration>,
//     /// Defines if websockets should be used (true) or TCP (false)
//     #[serde(rename = "useWs")]
//     pub use_ws: Option<bool>,
//     /// Defines the port to be used for the MQTT connection
//     pub port: Option<u16>,
//     /// Defines the maximum reconnection attempts before it returns an error
//     #[serde(rename = "maxReconnectionAttempts")]
//     pub max_reconnection_attempts: Option<usize>,
// }
//
// impl From<BrokerOptions> for iota_client::BrokerOptions {
//     fn from(value: BrokerOptions) -> iota_client::BrokerOptions {
//         let mut options = iota_client::BrokerOptions::new();
//         if let Some(automatic_disconnect) = value.automatic_disconnect {
//             options = options.automatic_disconnect(automatic_disconnect);
//         }
//         if let Some(timeout) = value.timeout {
//             options = options.timeout(timeout);
//         }
//         if let Some(use_ws) = value.use_ws {
//             options = options.use_ws(use_ws);
//         }
//         if let Some(port) = value.port {
//             options = options.port(port);
//         }
//         if let Some(max_reconnection_attempts) = value.max_reconnection_attempts {
//             options = options.max_reconnection_attempts(max_reconnection_attempts);
//         }
//         options
//     }
// }
//
// fn default_mqtt_enabled() -> bool {
//     true
// }
