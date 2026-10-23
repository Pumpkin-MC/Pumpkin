use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::MOVE_PLAYER_STATUS_ONLY;
use pumpkin_macros::java_packet;

#[java_packet(MOVE_PLAYER_STATUS_ONLY)]
pub struct SSetPlayerGround {
    pub on_ground: bool,
}

impl<'a> ServerPacket<'a> for SSetPlayerGround {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            on_ground: bytebuf.get_bool()?,
        })
    }
}

impl crate::ClientPacket for SSetPlayerGround {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_bool(self.on_ground)?;
        Ok(())
    }
}
