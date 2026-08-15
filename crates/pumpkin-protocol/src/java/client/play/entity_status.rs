use pumpkin_data::packet::clientbound::PLAY_ENTITY_EVENT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sends a status update for a specific entity.
///
/// This packet is a "catch-all" for various entity triggers that don't
/// warrant a complex packet of their own. It primarily handles visual
/// and logical state triggers, such as tool breaking, totem usage,
/// or sheep shearing.
#[java_packet(PLAY_ENTITY_EVENT)]
pub struct CEntityStatus {
    /// The Entity ID of the entity affected by the status change.
    pub entity_id: i32,
    /// The ID of the status/event to trigger.
    /// See the table below for common entity statuses.
    pub entity_status: i8,
}

impl CEntityStatus {
    #[must_use]
    pub const fn new(entity_id: i32, entity_status: i8) -> Self {
        Self {
            entity_id,
            entity_status,
        }
    }
}

impl ClientPacket for CEntityStatus {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i32_be(self.entity_id)?;
        write.write_i8(self.entity_status)?;
        Ok(())
    }
}
