use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError},
};
use pumpkin_data::packet::clientbound::play::TAKE_ITEM_ENTITY;
use pumpkin_macros::java_packet;

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
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&self.collector_entity_id)?;
        write.write_var_int(&self.stack_amount)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CTakeItemEntity {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let (entity_id, collector_entity_id) = (bytebuf.get_var_int()?, bytebuf.get_var_int()?);
        let stack_amount = bytebuf.get_var_int()?;
        Ok(Self {
            entity_id,
            collector_entity_id,
            stack_amount,
        })
    }
}
