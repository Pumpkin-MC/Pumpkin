use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::ser::{NetworkReadExt, ReadingError};
use crate::{ClientPacket, ServerPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::play::CONTAINER_SET_SLOT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONTAINER_SET_SLOT)]
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_container_id(&VarInt(i32::from(self.window_id)), version)?;
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            write.write_var_int(&self.state_id)?;
        }
        write.write_i16_be(self.slot)?;
        self.slot_data.write_with_version(&mut write, version)?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetContainerSlot<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id(version)?.0 as i8;
        let state_id = if *version >= JavaMinecraftVersion::V_1_17_1 {
            bytebuf.get_var_int()?
        } else {
            VarInt(0)
        };
        let slot = bytebuf.get_i16_be()?;
        let slot_data = ItemStackSerializer::read_with_version(bytebuf, version)?;
        Ok(Self {
            window_id,
            state_id,
            slot,
            slot_data: Box::leak(Box::new(slot_data)),
        })
    }
}
