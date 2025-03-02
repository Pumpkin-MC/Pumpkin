use pumpkin_data::packet::clientbound::PLAY_UPDATE_MOB_EFFECT;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_int::VarInt;

#[derive(Serialize)]
#[packet(PLAY_UPDATE_MOB_EFFECT)]
pub struct CUpdateMobEffect {
    entity_id: VarInt,
    effect_id: VarInt,
    amplifier: VarInt,
    duration: VarInt,
    flags: i8,
}

impl CUpdateMobEffect {
    pub fn new(
        entity_id: VarInt,
        effect_id: VarInt,
        amplifier: VarInt,
        duration: VarInt,
        flags: i8,
    ) -> Self {
        Self {
            entity_id,
            effect_id,
            amplifier,
            duration,
            flags,
        }
    }
}
