use pumpkin_data::packet::clientbound::PLAY_SET_HEALTH;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_SET_HEALTH)]
pub struct CSetHealth {
    pub health: f32,
    pub food: VarInt,
    pub food_saturation: f32,
}

impl CSetHealth {
    pub fn new(health: f32, food: VarInt, food_saturation: f32) -> Self {
        Self {
            health,
            food,
            food_saturation,
        }
    }
}
