use std::io::Write;

use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError};

use pumpkin_data::packet::clientbound::PLAY_SET_CURSOR_ITEM;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

#[java_packet(PLAY_SET_CURSOR_ITEM)]
pub struct CSetCursorItem<'a> {
    pub stack: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetCursorItem<'a> {
    #[must_use]
    pub const fn new(stack: &'a ItemStackSerializer<'a>) -> Self {
        Self { stack }
    }
}

impl ClientPacket for CSetCursorItem<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        self.stack.write_with_version(write, version)
    }
}
