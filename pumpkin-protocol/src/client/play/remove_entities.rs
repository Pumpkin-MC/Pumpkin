use pumpkin_data::packet::clientbound::PLAY_REMOVE_ENTITIES;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_REMOVE_ENTITIES)]
pub struct CRemoveEntities<'a> {
    count: VarInt,
    entity_ids: &'a [VarInt],
}

impl<'a> CRemoveEntities<'a> {
    pub fn new(entity_ids: &'a [VarInt]) -> Self {
        Self {
            count: VarInt(entity_ids.len() as i32),
            entity_ids,
        }
    }
}
