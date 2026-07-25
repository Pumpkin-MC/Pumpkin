use pumpkin_data::packet::clientbound::PLAY_TICKING_STATE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_TICKING_STATE)]
pub struct CTickingState {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

impl CTickingState {
    #[must_use]
    pub const fn new(tick_rate: f32, is_frozen: bool) -> Self {
        Self {
            tick_rate,
            is_frozen,
        }
    }
}

impl ClientPacket for CTickingState {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32_be(self.tick_rate)?;
        write.write_bool(self.is_frozen)?;
        Ok(())
    }
}
