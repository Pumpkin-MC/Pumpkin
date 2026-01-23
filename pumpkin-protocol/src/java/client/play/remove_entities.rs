use pumpkin_data::packet::clientbound::PLAY_REMOVE_ENTITIES;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_REMOVE_ENTITIES)]
pub struct CRemoveEntities<'a> {
    pub entity_ids: &'a [VarInt],
}

impl<'a> CRemoveEntities<'a> {
    #[must_use]
    pub const fn new(entity_ids: &'a [VarInt]) -> Self {
        Self { entity_ids }
    }
}
