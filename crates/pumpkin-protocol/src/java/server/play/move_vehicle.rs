use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_MOVE_VEHICLE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_MOVE_VEHICLE)]
pub struct SMoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl<'a> ServerPacket<'a> for SMoveVehicle {
    fn read(
        bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self {
            x: bytebuf.get_f64_be()?,
            y: bytebuf.get_f64_be()?,
            z: bytebuf.get_f64_be()?,
            yaw: bytebuf.get_f32_be()?,
            pitch: bytebuf.get_f32_be()?,
        })
    }
}
