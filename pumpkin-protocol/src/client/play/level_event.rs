use pumpkin_data::packet::clientbound::PLAY_LEVEL_EVENT;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_LEVEL_EVENT)]
pub struct CLevelEvent {
    event: i32,
    location: BlockPos,
    data: i32,
    disable_relative_volume: bool,
}

impl CLevelEvent {
    pub fn new(event: i32, location: BlockPos, data: i32, disable_relative_volume: bool) -> Self {
        Self {
            event,
            location,
            data,
            disable_relative_volume,
        }
    }
}
