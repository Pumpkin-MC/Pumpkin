use pumpkin_data::packet::clientbound::PLAY_SET_EXPERIENCE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_EXPERIENCE)]
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32(self.progress)?;
        write.write_var_int(&self.level)?;
        write.write_var_int(&self.total_experience)?;
        Ok(())
    }
}
