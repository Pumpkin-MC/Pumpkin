use pumpkin_data::packet::clientbound::PLAY_LEVEL_EVENT;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_LEVEL_EVENT)]
pub struct CWorldEvent<'a> {
    event: i32,
    location: &'a BlockPos,
    data: i32,
    disable_relative_volume: bool,
}

impl<'a> CWorldEvent<'a> {
    pub fn new(
        event: i32,
        location: &'a BlockPos,
        data: i32,
        disable_relative_volume: bool,
    ) -> Self {
        Self {
            event,
            location,
            data,
            disable_relative_volume,
        }
    }
}
