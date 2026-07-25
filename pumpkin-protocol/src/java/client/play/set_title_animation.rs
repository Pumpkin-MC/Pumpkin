use pumpkin_data::packet::clientbound::PLAY_SET_TITLES_ANIMATION;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_TITLES_ANIMATION)]
pub struct CTitleAnimation {
    pub fade_in_ticks: i32,
    pub stay_ticks: i32,
    pub fade_out_ticks: i32,
}

impl CTitleAnimation {
    #[must_use]
    pub const fn new(fade_in_ticks: i32, stay_ticks: i32, fade_out_ticks: i32) -> Self {
        Self {
            fade_in_ticks,
            stay_ticks,
            fade_out_ticks,
        }
    }
}

impl ClientPacket for CTitleAnimation {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i32_be(self.fade_in_ticks)?;
        write.write_i32_be(self.stay_ticks)?;
        write.write_i32_be(self.fade_out_ticks)?;
        Ok(())
    }
}
