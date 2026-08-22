use pumpkin_data::packet::clientbound::play::SET_EXPERIENCE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_EXPERIENCE)]
pub struct CSetExperience {
    pub progress: f32,
    pub level: VarInt,
    pub total_experience: VarInt,
}

impl CSetExperience {
    #[must_use]
    pub const fn new(progress: f32, level: VarInt, total_experience: VarInt) -> Self {
        Self {
            progress,
            level,
            total_experience,
        }
    }
}

impl ClientPacket for CSetExperience {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32_be(self.progress)?;
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i16_be(self.level.0 as i16)?;
            write.write_i16_be(self.total_experience.0 as i16)?;
        } else {
            write.write_var_int(&self.level)?;
            write.write_var_int(&self.total_experience)?;
        }
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for CSetExperience {
    fn read(
        bytebuf: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ser::ReadingError> {
        use crate::ser::NetworkReadExt;
        let progress = bytebuf.get_f32_be()?;
        let (level, total_experience) = if *version <= JavaMinecraftVersion::V_1_7_6 {
            (
                VarInt(i32::from(bytebuf.get_i16_be()?)),
                VarInt(i32::from(bytebuf.get_i16_be()?)),
            )
        } else {
            (bytebuf.get_var_int()?, bytebuf.get_var_int()?)
        };

        Ok(Self {
            progress,
            level,
            total_experience,
        })
    }
}
