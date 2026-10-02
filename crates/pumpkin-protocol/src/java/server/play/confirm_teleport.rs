use crate::{
    MultiVersionJavaPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
    // Only sent since 26.3
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
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
        // Since 26.3 the client echoes back where it accepted the teleport
        let (position, yaw, pitch) = if *version >= JavaMinecraftVersion::V_26_3 {
            (
                Vector3::new(
                    bytebuf.get_f64_be()?,
                    bytebuf.get_f64_be()?,
                    bytebuf.get_f64_be()?,
                ),
                bytebuf.get_f32_be()?,
                bytebuf.get_f32_be()?,
            )
        } else {
            (Vector3::new(0.0, 0.0, 0.0), 0.0, 0.0)
        };

        Ok(Self {
            teleport_id,
            position,
            yaw,
            pitch,
        })
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
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
        }
        Ok(())
    }
}
