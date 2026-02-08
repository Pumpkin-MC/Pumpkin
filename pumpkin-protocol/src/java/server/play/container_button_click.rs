use pumpkin_data::packet::serverbound::PLAY_CONTAINER_BUTTON_CLICK;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player clicks a button in a container
/// (e.g. enchanting table, stonecutter, loom, lectern).
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_CONTAINER_BUTTON_CLICK)]
pub struct SContainerButtonClick {
    pub container_id: VarInt,
    pub button_id: VarInt,
}
