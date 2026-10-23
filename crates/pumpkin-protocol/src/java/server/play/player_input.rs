use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::PLAYER_INPUT;
use pumpkin_macros::java_packet;

#[java_packet(PLAYER_INPUT)]
pub struct SPlayerInput {
    // Yep, exactly how it looks like
    pub input: i8,
}

impl SPlayerInput {
    pub const FORWARD: i8 = 1;
    pub const BACKWARD: i8 = 2;
    pub const LEFT: i8 = 4;
    pub const RIGHT: i8 = 8;
    pub const JUMP: i8 = 16;
    pub const SNEAK: i8 = 32;
    pub const SPRINT: i8 = 64;
}

impl<'a> ServerPacket<'a> for SPlayerInput {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            input: bytebuf.get_i8()?,
        })
    }
}

impl crate::ClientPacket for SPlayerInput {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i8(self.input)?;
        Ok(())
    }
}
