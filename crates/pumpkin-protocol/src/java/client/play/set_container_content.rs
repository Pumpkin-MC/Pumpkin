use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::ser::{NetworkReadExt, ReadingError};
use crate::{ClientPacket, ServerPacket, WritingError, ser::NetworkWriteExt};

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

impl<'a> ServerPacket<'a> for CSetContainerContent<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id(version)?;
        let state_id = if *version >= JavaMinecraftVersion::V_1_17_1 {
            bytebuf.get_var_int()?
        } else {
            VarInt(0)
        };
        let count = if *version >= JavaMinecraftVersion::V_1_17_1 {
            bytebuf.get_var_int()?.0
        } else {
            i32::from(bytebuf.get_i16_be()?)
        };
        if !(0..=4096).contains(&count) {
            return Err(ReadingError::Message("Slot count out of bounds".into()));
        }
        let mut slot_data = Vec::with_capacity(count as usize);
        for _ in 0..count {
            slot_data.push(ItemStackSerializer::read_with_version(bytebuf, version)?);
        }
        let carried_item = if *version >= JavaMinecraftVersion::V_1_17_1 {
            ItemStackSerializer::read_with_version(bytebuf, version)?
        } else {
            ItemStackSerializer(std::borrow::Cow::Borrowed(
                pumpkin_data::item_stack::ItemStack::EMPTY,
            ))
        };
        Ok(Self {
            window_id,
            state_id,
            slot_data: Box::leak(slot_data.into_boxed_slice()),
            carried_item: Box::leak(Box::new(carried_item)),
        })
    }
}
