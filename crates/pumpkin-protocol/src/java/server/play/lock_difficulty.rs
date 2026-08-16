use pumpkin_data::packet::serverbound::PLAY_LOCK_DIFFICULTY;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_LOCK_DIFFICULTY)]
pub struct SLockDifficulty {
    pub locked: bool,
}

impl<'a> ServerPacket<'a> for SLockDifficulty {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            locked: bytebuf.get_bool()?,
        })
    }
}
