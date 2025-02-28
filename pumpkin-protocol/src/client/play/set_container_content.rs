use crate::VarInt;
use crate::codec::slot::Slot;

use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_CONTENT;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CONTAINER_SET_CONTENT)]
pub struct CSetContainerContent<'a> {
    window_id: VarInt,
    state_id: VarInt,
    count: VarInt,
    slot_data: &'a [Slot],
    carried_item: &'a Slot,
}

impl<'a> CSetContainerContent<'a> {
    pub fn new(
        window_id: VarInt,
        state_id: VarInt,
        slots: &'a [Slot],
        carried_item: &'a Slot,
    ) -> Self {
        Self {
            window_id,
            state_id,
            count: slots.len().into(),
            slot_data: slots,
            carried_item,
        }
    }
}
