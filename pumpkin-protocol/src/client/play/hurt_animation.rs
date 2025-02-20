use pumpkin_data::packet::clientbound::PLAY_HURT_ANIMATION;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(PLAY_HURT_ANIMATION)]
pub struct CHurtAnimation<'a> {
    entity_id: &'a VarInt,
    yaw: f32,
}

impl<'a> CHurtAnimation<'a> {
    pub fn new(entity_id: &'a VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}
