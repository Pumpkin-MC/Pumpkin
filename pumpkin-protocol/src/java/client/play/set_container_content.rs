use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_CONTENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

#[java_packet(PLAY_CONTAINER_SET_CONTENT)]
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
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.window_id)?;
        write.write_var_int(&self.state_id)?;
        let slot_count = i32::try_from(self.slot_data.len()).map_err(|_| {
            WritingError::Message(format!(
                "{} slot entries do not fit in VarInt",
                self.slot_data.len()
            ))
        })?;
        write.write_var_int(&VarInt(slot_count))?;
        for stack in self.slot_data {
            stack.write_with_version(&mut write, version)?;
        }
        self.carried_item.write_with_version(&mut write, version)?;

        Ok(())
    }
}
