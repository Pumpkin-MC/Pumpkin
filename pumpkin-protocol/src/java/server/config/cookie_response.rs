use std::io::Read;

use pumpkin_data::packet::serverbound::CONFIG_COOKIE_RESPONSE;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

#[packet(CONFIG_COOKIE_RESPONSE)]
/// Response to a `CCookieRequest` (configuration) from the server.
/// The Notchian (vanilla) server only accepts responses of up to 5 KiB in size.
pub struct SConfigCookieResponse {
    pub key: ResourceLocation,
    pub has_payload: bool,
    pub payload: Option<Box<[u8]>>, // 5120,
}

const MAX_COOKIE_LENGTH: usize = 5120;

impl ServerPacket for SConfigCookieResponse {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let key = read.get_resource_location()?;
        let has_payload = read.get_bool()?;

        if !has_payload {
            return Ok(Self {
                key,
                has_payload,
                payload: None,
            });
        }

        let payload_length = read.get_var_int()?.0 as usize;
        if payload_length > MAX_COOKIE_LENGTH {
            return Err(ReadingError::TooLarge("SConfigCookieResponse".to_string()));
        }

        let payload = read.read_boxed_slice(payload_length)?;
        Ok(Self {
            key,
            has_payload,
            payload: Some(payload),
        })
    }
}
