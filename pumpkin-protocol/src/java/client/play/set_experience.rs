use pumpkin_data::packet::clientbound::PLAY_SET_EXPERIENCE;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_SET_EXPERIENCE)]
pub struct CSetExperience {
    pub progress: f32,
    pub total_experience: VarInt,
    pub level: VarInt,
}

impl CSetExperience {
    pub fn new(progress: f32, level: VarInt, total_experience: VarInt) -> Self {
        Self {
            progress,
            level,
            total_experience,
        }
    }
}
