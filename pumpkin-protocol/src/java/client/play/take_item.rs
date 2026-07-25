use crate::VarInt;
use pumpkin_data::packet::clientbound::PLAY_TAKE_ITEM_ENTITY;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_TAKE_ITEM_ENTITY)]
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&self.collector_entity_id)?;
        write.write_var_int(&self.stack_amount)?;
        Ok(())
    }
}
