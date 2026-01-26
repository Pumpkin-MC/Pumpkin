use pumpkin_data::packet::serverbound::PLAY_CONTAINER_CLOSE;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[java_packet(PLAY_CONTAINER_CLOSE)]
pub struct SCloseContainer {
    pub window_id: VarInt,
}
