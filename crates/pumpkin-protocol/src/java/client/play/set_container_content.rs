use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::ser::{NetworkReadExt, ReadingError};
use crate::{ClientPacket, ServerPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::play::CONTAINER_SET_CONTENT;
use pumpkin_macros::java_packet;

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
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_container_id(&self.window_id)?;
        write.write_var_int(&self.state_id)?;
        {
            let slot_count = i32::try_from(self.slot_data.len()).map_err(|_| {
                WritingError::Message(format!(
                    "{} slot entries do not fit in VarInt",
                    self.slot_data.len()
                ))
            })?;
            write.write_var_int(&VarInt(slot_count))?;
        };
        for stack in self.slot_data {
            stack.write(&mut write)?;
        }
        self.carried_item.write(&mut write)?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetContainerContent<'a> {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id()?;
        let state_id = bytebuf.get_var_int()?;
        let count = bytebuf.get_var_int()?.0;
        if !(0..=4096).contains(&count) {
            return Err(ReadingError::Message("Slot count out of bounds".into()));
        }
        let mut slot_data = Vec::with_capacity(count as usize);
        for _ in 0..count {
            slot_data.push(ItemStackSerializer::read(bytebuf)?);
        }
        let carried_item = ItemStackSerializer::read(bytebuf)?;
        Ok(Self {
            window_id,
            state_id,
            slot_data: Box::leak(slot_data.into_boxed_slice()),
            carried_item: Box::leak(Box::new(carried_item)),
        })
    }
}
