use pumpkin_data::packet::serverbound::PLAY_SET_BEACON;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player selects effects on a beacon.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_SET_BEACON)]
pub struct SSetBeacon {
    pub primary_effect: Option<VarInt>,
    pub secondary_effect: Option<VarInt>,
}
