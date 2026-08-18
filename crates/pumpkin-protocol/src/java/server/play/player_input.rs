use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_PLAYER_INPUT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PLAYER_INPUT)]
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
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            Ok(Self {
                input: bytebuf.get_i8()?,
            })
        } else {
            let sideways = bytebuf.get_f32_be()?;
            let forward = bytebuf.get_f32_be()?;
            let jumping = bytebuf.get_bool()?;
            let sneaking = bytebuf.get_bool()?;

            let mut input: i8 = 0;
            if forward > 0.0 {
                input |= Self::FORWARD;
            } else if forward < 0.0 {
                input |= Self::BACKWARD;
            }
            if sideways > 0.0 {
                input |= Self::LEFT;
            } else if sideways < 0.0 {
                input |= Self::RIGHT;
            }
            if jumping {
                input |= Self::JUMP;
            }
            if sneaking {
                input |= Self::SNEAK;
            }

            Ok(Self { input })
        }
    }
}

impl crate::ClientPacket for SPlayerInput {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_i8(self.input)?;
        } else {
            let mut sideways: f32 = 0.0;
            let mut forward: f32 = 0.0;
            if (self.input & Self::FORWARD) != 0 {
                forward += 1.0;
            }
            if (self.input & Self::BACKWARD) != 0 {
                forward -= 1.0;
            }
            if (self.input & Self::LEFT) != 0 {
                sideways += 1.0;
            }
            if (self.input & Self::RIGHT) != 0 {
                sideways -= 1.0;
            }
            let jumping = (self.input & Self::JUMP) != 0;
            let sneaking = (self.input & Self::SNEAK) != 0;
            write.write_f32_be(sideways)?;
            write.write_f32_be(forward)?;
            write.write_bool(jumping)?;
            write.write_bool(sneaking)?;
        }
        Ok(())
    }
}
