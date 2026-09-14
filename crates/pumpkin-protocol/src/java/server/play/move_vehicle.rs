use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::MOVE_VEHICLE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(MOVE_VEHICLE)]
pub struct SMoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl<'a> ServerPacket<'a> for SMoveVehicle {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let x = bytebuf.get_f64_be()?;
        let y = bytebuf.get_f64_be()?;
        let z = bytebuf.get_f64_be()?;
        let yaw = bytebuf.get_f32_be()?;
        let pitch = bytebuf.get_f32_be()?;
        let on_ground = if *version >= JavaMinecraftVersion::V_1_21_4 {
            bytebuf.get_bool()?
        } else {
            false
        };

        Ok(Self {
            x,
            y,
            z,
            yaw,
            pitch,
            on_ground,
        })
    }
}

impl crate::ClientPacket for SMoveVehicle {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_f64_be(self.x)?;
        write.write_f64_be(self.y)?;
        write.write_f64_be(self.z)?;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        if *version >= JavaMinecraftVersion::V_1_21_4 {
            write.write_bool(self.on_ground)?;
        }
        Ok(())
    }
}
