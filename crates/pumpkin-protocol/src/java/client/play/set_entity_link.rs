use pumpkin_data::packet::clientbound::play::SET_ENTITY_LINK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

/// Sent by the server to attach (leash) or detach an entity to another entity (e.g. leash knot, player, mob).
#[java_packet(SET_ENTITY_LINK)]
pub struct CSetEntityLink {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
    pub leash: bool,
}

impl CSetEntityLink {
    #[must_use]
    pub const fn new(attached_entity_id: i32, holding_entity_id: i32, leash: bool) -> Self {
        Self {
            attached_entity_id,
            holding_entity_id,
            leash,
        }
    }
}

impl ClientPacket for CSetEntityLink {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i32_be(self.attached_entity_id)?;
        write.write_i32_be(self.holding_entity_id)?;
        if *version <= JavaMinecraftVersion::V_1_8 {
            write.write_bool(self.leash)?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetEntityLink {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let attached_entity_id = bytebuf.get_i32_be()?;
        let holding_entity_id = bytebuf.get_i32_be()?;
        let leash = if *version <= JavaMinecraftVersion::V_1_8 {
            bytebuf.get_u8()? == 1
        } else {
            true
        };
        Ok(Self {
            attached_entity_id,
            holding_entity_id,
            leash,
        })
    }
}
