use crate::codec::item_stack_seralizer::ItemStackSerializer;

use pumpkin_data::packet::clientbound::PLAY_SET_CURSOR_ITEM;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SET_CURSOR_ITEM)]
pub struct CSetCursorItem<'a> {
    pub stack: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetCursorItem<'a> {
    #[must_use]
    pub fn new(stack: &'a ItemStackSerializer<'a>) -> Self {
        Self { stack }
    }
}
