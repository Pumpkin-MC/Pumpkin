use pumpkin_data::packet::clientbound::PLAY_SET_HEALTH;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(PLAY_SET_HEALTH)]
pub struct CSetHealth {
    health: f32,
    food: VarInt,
    food_saturation: f32,
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
