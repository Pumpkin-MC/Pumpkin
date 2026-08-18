use pumpkin_data::packet::clientbound::PLAY_SERVER_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;

use crate::{ClientPacket, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SERVER_DATA)]
pub struct CServerData<'a> {
    pub motd: &'a TextComponent,
    pub icon_base64: Option<&'a str>,
}

impl<'a> CServerData<'a> {
    #[must_use]
    pub const fn new(motd: &'a TextComponent, icon_base64: Option<&'a str>) -> Self {
        Self { motd, icon_base64 }
    }
}

impl ClientPacket for CServerData<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.motd.encode())?;
        if let Some(icon) = self.icon_base64 {
            write.write_bool(true)?;
            write.write_string(icon)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
