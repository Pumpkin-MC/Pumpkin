use pumpkin_data::packet::clientbound::play::SERVER_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;

use crate::{ClientPacket, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SERVER_DATA)]
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_1_19_4 {
            write.write_component(self.motd, version)?;
            if let Some(icon) = self.icon_base64 {
                write.write_bool(true)?;
                let raw_b64 = icon.strip_prefix("data:image/png;base64,").unwrap_or(icon);
                if let Ok(bytes) = pumpkin_util::jwt::decode_b64_standard(raw_b64) {
                    write.write_slice(&bytes)?;
                } else {
                    write.write_slice(&[])?;
                }
            } else {
                write.write_bool(false)?;
            }
        } else {
            write.write_bool(true)?;
            write.write_component(self.motd, version)?;
            if let Some(icon) = self.icon_base64 {
                write.write_bool(true)?;
                write.write_string(icon)?;
            } else {
                write.write_bool(false)?;
            }
            if *version < JavaMinecraftVersion::V_1_19_3 {
                write.write_bool(false)?;
            }
            if *version >= JavaMinecraftVersion::V_1_19_1
                && *version < JavaMinecraftVersion::V_1_20_5
            {
                write.write_bool(false)?;
            }
        }
        Ok(())
    }
}
