use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_MOVE_PLAYER_ROT)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}

impl<'a> ServerPacket<'a> for SPlayerRotation {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            yaw: bytebuf.get_f32_be()?,
            pitch: bytebuf.get_f32_be()?,
            ground: bytebuf.get_bool()?,
        })
    }
}

impl crate::ClientPacket for SPlayerRotation {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        write.write_bool(self.ground)?;
        Ok(())
    }
}
