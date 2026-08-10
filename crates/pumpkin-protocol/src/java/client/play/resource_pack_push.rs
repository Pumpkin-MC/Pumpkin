use pumpkin_data::packet::clientbound::PLAY_RESOURCE_PACK_PUSH;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

use crate::{
    ClientPacket,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_RESOURCE_PACK_PUSH)]
pub struct CPlayResourcePackPush<'a> {
    pub uuid: &'a uuid::Uuid,
    pub url: &'a str,
    pub hash: &'a str,
    pub forced: bool,
    pub prompt_message: Option<TextComponent>,
}

impl<'a> CPlayResourcePackPush<'a> {
    #[must_use]
    pub const fn new(uuid: &'a uuid::Uuid, url: &'a str, hash: &'a str) -> Self {
        Self {
            uuid,
            url,
            hash,
            forced: false,
            prompt_message: None,
        }
    }
}

impl ClientPacket for CPlayResourcePackPush<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_uuid(self.uuid)?;
        write.write_string(self.url)?;
        write.write_string(self.hash)?;
        write.write_bool(self.forced)?;
        write.write_bool(self.prompt_message.is_some())?;
        if let Some(prompt) = &self.prompt_message {
            write.write_slice(&prompt.encode())?;
        }
        Ok(())
    }
}
