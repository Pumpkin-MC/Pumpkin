use pumpkin_data::packet::clientbound::PLAY_UPDATE_MOB_EFFECT;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_int::VarInt;

#[derive(Serialize)]
#[packet(PLAY_UPDATE_MOB_EFFECT)]
pub struct CUpdateMobEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
    pub amplifier: VarInt,
    pub duration: VarInt,
    pub flags: i8,
}

impl CUpdateMobEffect {
    #[must_use]
    pub const fn new(
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
