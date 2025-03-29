use pumpkin_data::packet::serverbound::PLAY_ACCEPT_TELEPORTATION;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[packet(PLAY_ACCEPT_TELEPORTATION)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
