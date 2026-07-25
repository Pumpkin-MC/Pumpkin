use pumpkin_data::packet::clientbound::PLAY_TAB_LIST;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Updates the header and footer of the player list (Tab List).
#[java_packet(PLAY_TAB_LIST)]
pub struct CTabList<'a> {
    /// The text to be displayed at the top of the player list.
    pub header: &'a TextComponent,
    /// The text to be displayed at the bottom of the player list.
    pub footer: &'a TextComponent,
}

impl<'a> CTabList<'a> {
    #[must_use]
    pub const fn new(header: &'a TextComponent, footer: &'a TextComponent) -> Self {
        Self { header, footer }
    }
}

impl ClientPacket for CTabList<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.header.encode())?;
        write.write_slice(&self.footer.encode())?;
        Ok(())
    }
}
