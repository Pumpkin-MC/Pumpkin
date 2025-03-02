use pumpkin_data::packet::serverbound::PLAY_CONTAINER_CLOSE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[packet(PLAY_CONTAINER_CLOSE)]
pub struct SCloseContainer {
    pub window_id: VarInt,
}
