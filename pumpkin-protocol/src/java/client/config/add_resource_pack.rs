use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;

use pumpkin_data::packet::clientbound::CONFIG_RESOURCE_PACK_PUSH;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_RESOURCE_PACK_PUSH)]
pub struct CConfigAddResourcePack<'a> {
    pub uuid: &'a uuid::Uuid,
    pub url: &'a str,
    pub hash: &'a str, // max 40
    pub forced: bool,
    pub prompt_message: Option<TextComponent>,
}

impl<'a> CConfigAddResourcePack<'a> {
    #[must_use]
    pub const fn new(
        uuid: &'a uuid::Uuid,
        url: &'a str,
        hash: &'a str,
        forced: bool,
        prompt_message: Option<TextComponent>,
    ) -> Self {
        Self {
            uuid,
            url,
            hash,
            forced,
            prompt_message,
        }
    }
}

impl ClientPacket for CConfigAddResourcePack<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_uuid(self.uuid)?;
        write.write_string(self.url)?;
        write.write_string(self.hash)?;
        write.write_bool(self.forced)?;
        if let Some(_prompt) = &self.prompt_message {
            write.write_bool(true)?;
            // TODO
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
