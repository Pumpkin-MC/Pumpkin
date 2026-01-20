use pumpkin_data::packet::clientbound::PLAY_HURT_ANIMATION;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_HURT_ANIMATION)]
pub struct CHurtAnimation {
    pub entity_id: VarInt,
    pub yaw: f32,
}

impl CHurtAnimation {
    #[must_use] 
    pub const fn new(entity_id: VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}
