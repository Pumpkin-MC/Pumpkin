use pumpkin_data::packet::serverbound::PLAY_CONTAINER_CLOSE;
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet(PLAY_CONTAINER_CLOSE)]
pub struct SCloseContainer {
    pub window_id: VarInt,
}
