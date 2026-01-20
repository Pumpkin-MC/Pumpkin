use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;

use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_SLOT;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CONTAINER_SET_SLOT)]
pub struct CSetContainerSlot<'a> {
    pub window_id: i8,
    pub state_id: VarInt,
    pub slot: i16,
    pub slot_data: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetContainerSlot<'a> {
    #[must_use] 
    pub fn new(
        window_id: i8,
        state_id: i32,
        slot: i16,
        slot_data: &'a ItemStackSerializer<'a>,
    ) -> Self {
        Self {
            window_id,
            state_id: state_id.into(),
            slot,
            slot_data,
        }
    }
}
