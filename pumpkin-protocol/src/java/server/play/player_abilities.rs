use pumpkin_data::packet::serverbound::PLAY_PLAYER_ABILITIES;
use pumpkin_macros::java_packet;

// The vanilla client sends this packet when the player starts/stops flying. Bitmask 0x02 is set when the player is flying.

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PLAYER_ABILITIES)]
pub struct SPlayerAbilities {
    pub flags: i8,
}

impl<'a> ServerPacket<'a> for SPlayerAbilities {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            flags: bytebuf.get_i8()?,
        })
    }
}
