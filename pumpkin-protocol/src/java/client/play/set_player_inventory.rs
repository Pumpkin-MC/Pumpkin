use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::PLAY_SET_PLAYER_INVENTORY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

#[java_packet(PLAY_SET_PLAYER_INVENTORY)]
pub struct CSetPlayerInventory<'a> {
    pub slot: VarInt,
    pub item: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetPlayerInventory<'a> {
    #[must_use]
    pub const fn new(slot: VarInt, item: &'a ItemStackSerializer<'a>) -> Self {
        Self { slot, item }
    }
}

impl ClientPacket for CSetPlayerInventory<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_var_int(&self.slot)?;
        self.item.write_with_version(write, version)
    }
}
