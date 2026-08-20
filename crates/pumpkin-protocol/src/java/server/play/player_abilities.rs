use pumpkin_data::packet::serverbound::play::PLAYER_ABILITIES;
use pumpkin_macros::java_packet;

// The vanilla client sends this packet when the player starts/stops flying. Bitmask 0x02 is set when the player is flying.

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAYER_ABILITIES)]
pub struct SPlayerAbilities {
    pub flags: i8,
}

impl<'a> ServerPacket<'a> for SPlayerAbilities {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let flags = bytebuf.get_i8()?;
        if version < &JavaMinecraftVersion::V_1_16 {
            // Read fly_speed and walk_speed which were sent prior to 1.16
            let _fly_speed = bytebuf.get_f32_be()?;
            let _walk_speed = bytebuf.get_f32_be()?;
        }
        Ok(Self { flags })
    }
}

impl crate::ClientPacket for SPlayerAbilities {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i8(self.flags)?;
        if version < &JavaMinecraftVersion::V_1_16 {
            // Write default fly and walk speed if they aren't stored
            write.write_f32_be(0.05)?;
            write.write_f32_be(0.1)?;
        }
        Ok(())
    }
}
