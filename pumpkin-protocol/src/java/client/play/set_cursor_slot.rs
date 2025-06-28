use crate::codec::item_stack_seralizer::ItemStackSerializer;

use pumpkin_data::packet::clientbound::PLAY_SET_CURSOR_ITEM;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_CURSOR_ITEM)]
pub struct CSetCursorItem<'a> {
    stack: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetCursorItem<'a> {
    pub fn new(stack: &'a ItemStackSerializer<'a>) -> Self {
        Self { stack }
    }
}
