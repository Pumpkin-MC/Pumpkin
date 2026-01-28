use pumpkin_data::packet::clientbound::PLAY_SET_EXPERIENCE;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_SET_EXPERIENCE)]
pub struct CSetExperience {
    pub progress: f32,
    pub total_experience: VarInt,
    pub level: VarInt,
}

impl CSetExperience {
    #[must_use]
    pub fn new(progress: f32, level: VarInt, total_experience: VarInt) -> Self {
        Self {
            progress,
            total_experience,
            level,
        }
    }
}
