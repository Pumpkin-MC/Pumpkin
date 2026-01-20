use crate::codec::var_int::VarInt;
use pumpkin_data::packet::clientbound::PLAY_REMOVE_MOB_EFFECT;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_REMOVE_MOB_EFFECT)]
pub struct CRemoveMobEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
}

impl CRemoveMobEffect {
    #[must_use] 
    pub const fn new(entity_id: VarInt, effect_id: VarInt) -> Self {
        Self {
            entity_id,
            effect_id,
        }
    }
}
