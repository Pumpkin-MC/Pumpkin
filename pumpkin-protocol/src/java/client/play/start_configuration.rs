use pumpkin_data::packet::clientbound::PLAY_START_CONFIGURATION;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Tells the client to switch from Play state back to Configuration state.
///
/// The client must respond with a Configuration Acknowledged packet.
#[derive(Serialize)]
#[java_packet(PLAY_START_CONFIGURATION)]
pub struct CStartConfiguration;
