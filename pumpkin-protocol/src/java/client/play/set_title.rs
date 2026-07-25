use pumpkin_data::packet::clientbound::PLAY_SET_TITLE_TEXT;
use pumpkin_util::text::TextComponent;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_TITLE_TEXT)]
pub struct CTitleText<'a> {
    pub title: &'a TextComponent,
}

impl<'a> CTitleText<'a> {
    #[must_use]
    pub const fn new(title: &'a TextComponent) -> Self {
        Self { title }
    }
}

impl ClientPacket for CTitleText<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.title.encode())?;
        Ok(())
    }
}
