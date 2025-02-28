use crate::VarInt;
use pumpkin_data::packet::clientbound::PLAY_TAKE_ITEM_ENTITY;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_TAKE_ITEM_ENTITY)]
pub struct CTakeItemEntity {
    /// The Entity ID of the Item Entity
    entity_id: VarInt,
    /// The Entity ID of the Entity who is collecting the Item
    collector_entity_id: VarInt,
    /// The Number of items in the Stack
    stack_amount: VarInt,
}

impl CTakeItemEntity {
    pub fn new(entity_id: VarInt, collector_entity_id: VarInt, stack_amount: VarInt) -> Self {
        Self {
            entity_id,
            collector_entity_id,
            stack_amount,
        }
    }
}
