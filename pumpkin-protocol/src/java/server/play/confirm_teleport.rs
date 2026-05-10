use pumpkin_data::packet::serverbound::PLAY_ACCEPT_TELEPORTATION;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(serde::Deserialize, Serialize)]
#[java_packet(PLAY_ACCEPT_TELEPORTATION)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
