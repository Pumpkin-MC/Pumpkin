use pumpkin_data::packet::serverbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// This packet signals to the server that the client is ready to transition
/// from the `Configuration` state to the `Play` state.
#[derive(Serialize)]
#[java_packet(CONFIG_FINISH_CONFIGURATION)]
pub struct SAcknowledgeFinishConfig;
