use pumpkin_data::packet::clientbound::play::REMOVE_ENTITIES;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to instruct the client to remove (despawn) one or more entities.
///
/// This is typically sent when an entity leaves the player's tracking range,
/// is killed, or is otherwise removed from the world.
#[java_packet(REMOVE_ENTITIES)]
pub struct CRemoveEntities<'a> {
    /// A list of entity IDs to be removed.
    pub entity_ids: &'a [VarInt],
}

impl<'a> CRemoveEntities<'a> {
    #[must_use]
    pub const fn new(entity_ids: &'a [VarInt]) -> Self {
        Self { entity_ids }
    }
}

impl ClientPacket for CRemoveEntities<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version == JavaMinecraftVersion::V_1_17 {
            write.write_var_int(self.entity_ids.first().unwrap_or(&VarInt(0)))?;
        } else if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_u8(self.entity_ids.len() as u8)?;
            for id in self.entity_ids {
                write.write_i32_be(id.0)?;
            }
        } else {
            write.write_var_int(&VarInt(self.entity_ids.len() as i32))?;
            for id in self.entity_ids {
                write.write_var_int(id)?;
            }
        }
        Ok(())
    }
}
