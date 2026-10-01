use pumpkin_data::packet::serverbound::play::PLAYER_ABILITIES;
use pumpkin_macros::java_packet;

// The vanilla client sends this packet when the player starts/stops flying. Bitmask 0x02 is set when the player is flying.

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PLAYER_ABILITIES)]
pub struct SPlayerAbilities {
    pub flags: i8,
    pub fly_speed: Option<f32>,
    pub walk_speed: Option<f32>,
}

impl SPlayerAbilities {
    #[must_use]
    pub const fn new(flags: i8, fly_speed: Option<f32>, walk_speed: Option<f32>) -> Self {
        Self {
            flags,
            fly_speed,
            walk_speed,
        }
    }

    #[must_use]
    pub const fn is_invulnerable(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    #[must_use]
    pub const fn is_flying(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    #[must_use]
    pub const fn is_flight_allowed(&self) -> bool {
        (self.flags & 0x04) != 0
    }

    #[must_use]
    pub const fn is_creative_mode(&self) -> bool {
        (self.flags & 0x08) != 0
    }
}

impl<'a> ServerPacket<'a> for SPlayerAbilities {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let flags = bytebuf.get_i8()?;
        let (fly_speed, walk_speed) = (None, None);
        Ok(Self {
            flags,
            fly_speed,
            walk_speed,
        })
    }
}

impl crate::ClientPacket for SPlayerAbilities {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i8(self.flags)?;

        Ok(())
    }
}
