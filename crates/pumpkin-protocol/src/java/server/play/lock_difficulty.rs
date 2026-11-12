use pumpkin_data::packet::serverbound::play::LOCK_DIFFICULTY;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(LOCK_DIFFICULTY)]
pub struct SLockDifficulty {
    pub locked: bool,
}

impl<'a> ServerPacket<'a> for SLockDifficulty {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            locked: bytebuf.get_bool()?,
        })
    }
}

impl crate::ClientPacket for SLockDifficulty {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_bool(self.locked)?;
        Ok(())
    }
}
