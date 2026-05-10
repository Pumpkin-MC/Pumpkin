use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_SLOT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

#[java_packet(PLAY_CONTAINER_SET_SLOT)]
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

impl ClientPacket for CSetContainerSlot<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_i8(self.window_id)?;
        write.write_var_int(&self.state_id)?;
        write.write_i16_be(self.slot)?;
        self.slot_data.write_with_version(&mut write, version)?;

        Ok(())
    }
}
