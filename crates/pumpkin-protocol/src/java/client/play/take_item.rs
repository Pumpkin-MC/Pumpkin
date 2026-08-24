use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError},
};
use pumpkin_data::packet::clientbound::play::TAKE_ITEM_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(TAKE_ITEM_ENTITY)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CTakeItemEntity {
    /// The entity id of the item entity.
    pub entity_id: VarInt,
    /// The entity id of the entity who is collecting the item.
    pub collector_entity_id: VarInt,
    /// The Number of items in the Stack
    pub stack_amount: VarInt,
}

impl CTakeItemEntity {
    #[must_use]
    pub const fn new(entity_id: VarInt, collector_entity_id: VarInt, stack_amount: VarInt) -> Self {
        Self {
            entity_id,
            collector_entity_id,
            stack_amount,
        }
    }
}

impl ClientPacket for CTakeItemEntity {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
            write.write_i32_be(self.collector_entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
            write.write_var_int(&self.collector_entity_id)?;
        }
        if *version >= JavaMinecraftVersion::V_1_11 {
            write.write_var_int(&self.stack_amount)?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CTakeItemEntity {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let (entity_id, collector_entity_id) = if *version <= JavaMinecraftVersion::V_1_7_6 {
            (VarInt(bytebuf.get_i32_be()?), VarInt(bytebuf.get_i32_be()?))
        } else {
            (bytebuf.get_var_int()?, bytebuf.get_var_int()?)
        };
        let stack_amount = if *version >= JavaMinecraftVersion::V_1_11 {
            bytebuf.get_var_int()?
        } else {
            VarInt(1)
        };
        Ok(Self {
            entity_id,
            collector_entity_id,
            stack_amount,
        })
    }
}
