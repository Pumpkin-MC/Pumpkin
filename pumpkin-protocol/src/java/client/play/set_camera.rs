use pumpkin_data::packet::clientbound::PLAY_SET_CAMERA;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Sets the camera to view from a specific entity's perspective.
///
/// Used for spectator mode. Set to the player's own entity ID to reset.
#[derive(Serialize)]
#[java_packet(PLAY_SET_CAMERA)]
pub struct CSetCamera {
    pub camera_id: VarInt,
}

impl CSetCamera {
    #[must_use]
    pub const fn new(camera_id: VarInt) -> Self {
        Self { camera_id }
    }
}
