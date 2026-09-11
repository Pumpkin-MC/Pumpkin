use pumpkin_data::packet::clientbound::play::MOUNT_SCREEN_OPEN;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(MOUNT_SCREEN_OPEN)]
pub struct COpenMountScreen {
    pub window_id: u8,
    pub slot_count: VarInt,
    pub entity_id: i32,
}

impl COpenMountScreen {
    #[must_use]
    pub const fn new(window_id: u8, slot_count: VarInt, entity_id: i32) -> Self {
        Self {
            window_id,
            slot_count,
            entity_id,
        }
    }
}

impl ClientPacket for COpenMountScreen {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_u8(self.window_id)?;
        write.write_var_int(&self.slot_count)?;
        write.write_i32_be(self.entity_id)?;
        Ok(())
    }
}
