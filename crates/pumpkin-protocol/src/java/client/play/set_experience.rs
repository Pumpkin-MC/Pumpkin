use pumpkin_data::packet::clientbound::play::SET_EXPERIENCE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;

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
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32_be(self.progress)?;
        write.write_var_int(&self.level)?;
        write.write_var_int(&self.total_experience)?;
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for CSetExperience {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, crate::ser::ReadingError> {
        use crate::ser::NetworkReadExt;
        let progress = bytebuf.get_f32_be()?;
        let (level, total_experience) = (bytebuf.get_var_int()?, bytebuf.get_var_int()?);

        Ok(Self {
            progress,
            level,
            total_experience,
        })
    }
}
