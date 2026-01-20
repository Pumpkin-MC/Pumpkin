use serde::Serialize;

use pumpkin_data::packet::clientbound::PLAY_SET_DEFAULT_SPAWN_POSITION;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

#[derive(Serialize)]
#[packet(PLAY_SET_DEFAULT_SPAWN_POSITION)]
pub struct CPlayerSpawnPosition {
    pub dimension_name: String,
    pub location: BlockPos,
    pub yaw: f32,
    pub pitch: f32,
}

impl CPlayerSpawnPosition {
    #[must_use] 
    pub const fn new(location: BlockPos, yaw: f32, pitch: f32, dimension_name: String) -> Self {
        Self {
            dimension_name,
            location,
            yaw,
            pitch,
        }
    }
}
