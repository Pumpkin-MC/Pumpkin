use pumpkin_data::packet::clientbound::PLAY_RESET_SCORE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_RESET_SCORE)]
pub struct CResetScore {
    pub entity_name: String,
    pub objective_name: Option<String>,
}

impl CResetScore {
    #[must_use]
    pub const fn new(entity_name: String, objective_name: Option<String>) -> Self {
        Self {
            entity_name,
            objective_name,
        }
    }
}

impl ClientPacket for CResetScore {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(&self.entity_name)?;
        write.write_option(&self.objective_name, |w, obj| w.write_string(obj))?;
        Ok(())
    }
}
