use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::MOVE_VEHICLE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(MOVE_VEHICLE)]
pub struct CMoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl CMoveVehicle {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
        }
    }
}

impl ClientPacket for CMoveVehicle {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_f64_be(self.x)?;
        write.write_f64_be(self.y)?;
        write.write_f64_be(self.z)?;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CMoveVehicle {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            x: bytebuf.get_f64_be()?,
            y: bytebuf.get_f64_be()?,
            z: bytebuf.get_f64_be()?,
            yaw: bytebuf.get_f32_be()?,
            pitch: bytebuf.get_f32_be()?,
        })
    }
}
