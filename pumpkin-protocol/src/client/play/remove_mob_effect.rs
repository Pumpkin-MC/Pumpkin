use pumpkin_data::packet::clientbound::PLAY_REMOVE_MOB_EFFECT;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_int::VarInt;

#[derive(Serialize)]
#[packet(PLAY_REMOVE_MOB_EFFECT)]
pub struct CRemoveMobEffect {
    entity_id: VarInt,
    effect_id: VarInt,
}

impl CRemoveMobEffect {
    pub fn new(entity_id: VarInt, effect_id: VarInt) -> Self {
        Self {
            entity_id,
            effect_id,
        }
    }
}
