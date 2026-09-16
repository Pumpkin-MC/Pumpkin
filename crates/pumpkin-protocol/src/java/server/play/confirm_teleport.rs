use crate::{
    MultiVersionJavaPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
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
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let teleport_id = bytebuf.get_var_int()?;
        if *version >= JavaMinecraftVersion::V_26_3 {
            // 26.3 echoes the position (3 doubles) and rotation (2 floats) the client ended up at
            bytebuf.read_slice_borrowed(3 * 8 + 2 * 4)?;
        }
        Ok(Self { teleport_id })
    }
}

impl crate::ClientPacket for SConfirmTeleport {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.teleport_id)?;
        if *version >= JavaMinecraftVersion::V_26_3 {
            write.write_slice(&[0; 3 * 8 + 2 * 4])?;
        }
        Ok(())
    }
}
