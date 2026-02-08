use pumpkin_data::packet::serverbound::PLAY_CONFIGURATION_ACKNOWLEDGED;
use pumpkin_macros::java_packet;

/// Sent by the client to acknowledge the server's request to re-enter
/// Configuration state from Play state.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_CONFIGURATION_ACKNOWLEDGED)]
pub struct SConfigurationAcknowledged;
