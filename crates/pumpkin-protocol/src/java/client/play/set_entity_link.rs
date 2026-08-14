use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_LINK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{ClientPacket, ser::NetworkWriteExt};

/// Sent by the server to attach (leash) or detach an entity to another entity (e.g. leash knot, player, mob).
#[java_packet(PLAY_SET_ENTITY_LINK)]
pub struct CSetEntityLink {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
}

impl CSetEntityLink {
    #[must_use]
    pub const fn new(attached_entity_id: i32, holding_entity_id: i32) -> Self {
        Self {
            attached_entity_id,
            holding_entity_id,
        }
    }
}

impl ClientPacket for CSetEntityLink {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i32_be(self.attached_entity_id)?;
        write.write_i32_be(self.holding_entity_id)?;
        Ok(())
    }
}
