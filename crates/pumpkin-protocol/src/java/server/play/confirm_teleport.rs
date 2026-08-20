use crate::{
    MultiVersionJavaPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}

impl MultiVersionJavaPacket for SConfirmTeleport {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        if version >= JavaMinecraftVersion::V_1_9 {
            0
        } else {
            -1
        }
    }
}

impl<'a> ServerPacket<'a> for SConfirmTeleport {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            teleport_id: bytebuf.get_var_int()?,
        })
    }
}

impl crate::ClientPacket for SConfirmTeleport {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.teleport_id)?;
        Ok(())
    }
}
