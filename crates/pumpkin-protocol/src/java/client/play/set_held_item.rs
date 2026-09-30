use pumpkin_data::packet::clientbound::play::SET_HELD_SLOT;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ClientPacket;
use crate::packet::MultiVersionJavaPacket;
use crate::ser::NetworkWriteExt;

pub struct CSetSelectedSlot {
    pub slot: i8,
}

impl CSetSelectedSlot {
    #[must_use]
    pub const fn new(slot: i8) -> Self {
        Self { slot }
    }
}

impl ClientPacket for CSetSelectedSlot {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&crate::VarInt(i32::from(self.slot)))?;
        Ok(())
    }
}

impl MultiVersionJavaPacket for CSetSelectedSlot {
    fn to_id(_version: JavaMinecraftVersion) -> i32 {
        SET_HELD_SLOT.to_id()
    }
}
