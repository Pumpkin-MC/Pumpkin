use pumpkin_data::packet::serverbound::PLAY_ACCEPT_TELEPORTATION;
use pumpkin_macros::server_packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[server_packet(PLAY_ACCEPT_TELEPORTATION)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
