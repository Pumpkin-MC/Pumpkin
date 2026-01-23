use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;

use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_CONTENT;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CONTAINER_SET_CONTENT)]
pub struct CSetContainerContent<'a> {
    pub window_id: VarInt,
    pub state_id: VarInt,
    pub slot_data: &'a [ItemStackSerializer<'a>],
    pub carried_item: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetContainerContent<'a> {
    #[must_use]
    pub const fn new(
        window_id: VarInt,
        state_id: VarInt,
        slots: &'a [ItemStackSerializer],
        carried_item: &'a ItemStackSerializer,
    ) -> Self {
        Self {
            window_id,
            state_id,
            slot_data: slots,
            carried_item,
        }
    }
}
