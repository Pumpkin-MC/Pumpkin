use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_REMOVE_ENTITIES;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_REMOVE_ENTITIES)]
pub struct CRemoveEntities<'a> {
    entity_count: VarInt,
    entity_ids: Cow<'a, [VarInt]>,
}

impl<'a> CRemoveEntities<'a> {
    pub fn new(entity_ids: Cow<'a, [VarInt]>) -> Self {
        Self {
            entity_count: entity_ids.len().into(),
            entity_ids,
        }
    }
}
