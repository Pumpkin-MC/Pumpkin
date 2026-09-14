use std::io::Write;

use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError};

use pumpkin_data::packet::clientbound::play::SET_CURSOR_ITEM;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_CURSOR_ITEM)]
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
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        self.stack.write_with_version(&mut write, version)
    }
}
