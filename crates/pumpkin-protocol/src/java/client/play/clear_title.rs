use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_CLEAR_TITLES;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
/// Removes any currently displayed Title or Subtitle from the player's screen.
///
/// This packet is used to immediately hide titles that are currently in their
/// "stay" or "fade-out" phases.
#[java_packet(PLAY_CLEAR_TITLES)]
pub struct CClearTitle {
    /// If true, the client also resets the title timings (fade-in, stay, fade-out)
    /// to their default values (10, 70, 20 ticks).
    ///
    /// Set this to false if you want to clear the text but keep custom timings
    /// for the next title you send.
    pub reset: bool,
}

impl CClearTitle {
    #[must_use]
    pub const fn new(reset: bool) -> Self {
        Self { reset }
    }
}

impl ClientPacket for CClearTitle {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_bool(self.reset)?;
        Ok(())
    }
}
