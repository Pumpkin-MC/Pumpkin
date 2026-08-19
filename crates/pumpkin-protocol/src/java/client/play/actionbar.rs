use pumpkin_data::packet::clientbound::PLAY_SET_ACTION_BAR_TEXT;
use pumpkin_util::text::TextComponent;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
/// Updates the text displayed above the player's hotbar (the Action Bar).
///
/// Unlike chat messages, Action Bar text is transient and generally used for
/// non-critical status information like "Now entering: Wilderness" or
/// mana/stamina counters.
#[java_packet(PLAY_SET_ACTION_BAR_TEXT)]
pub struct CActionBar<'a> {
    /// The text component to be displayed.
    pub action_bar: &'a TextComponent,
}

impl<'a> CActionBar<'a> {
    #[must_use]
    pub const fn new(action_bar: &'a TextComponent) -> Self {
        Self { action_bar }
    }
}

impl ClientPacket for CActionBar<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.action_bar.encode())?;
        Ok(())
    }
}
