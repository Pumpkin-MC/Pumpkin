use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::play::CONTAINER_SET_CONTENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONTAINER_SET_CONTENT)]
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

impl ClientPacket for CSetContainerContent<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_container_id(&self.window_id, version)?;
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            write.write_var_int(&self.state_id)?;
        }
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            let slot_count = i32::try_from(self.slot_data.len()).map_err(|_| {
                WritingError::Message(format!(
                    "{} slot entries do not fit in VarInt",
                    self.slot_data.len()
                ))
            })?;
            write.write_var_int(&VarInt(slot_count))?;
        } else {
            let slot_count = i16::try_from(self.slot_data.len()).map_err(|_| {
                WritingError::Message(format!(
                    "{} slot entries do not fit in Short",
                    self.slot_data.len()
                ))
            })?;
            write.write_i16_be(slot_count)?;
        }
        for stack in self.slot_data {
            stack.write_with_version(&mut write, version)?;
        }
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            self.carried_item.write_with_version(&mut write, version)?;
        }

        Ok(())
    }
}
