use std::io::Read;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::LOGIN_COOKIE_RESPONSE;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

#[packet(LOGIN_COOKIE_RESPONSE)]
/// Response to a `CCookieRequest` (login) from the server.
/// The Notchian server only accepts responses of up to 5 kiB in size.
pub struct SLoginCookieResponse {
    pub key: ResourceLocation,
    pub payload: Option<Box<[u8]>>, // 5120,
}

const MAX_COOKIE_LENGTH: usize = 5120;

impl ServerPacket for SLoginCookieResponse {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        let key = read.get_resource_location()?;
        let has_payload = read.get_bool()?;

        if !has_payload {
            return Ok(Self { key, payload: None });
        }

        let payload_length = read.get_var_int()?;
        let length = payload_length.0 as usize;
        if length > MAX_COOKIE_LENGTH {
            return Err(ReadingError::TooLarge("SLoginCookieResponse".to_string()));
        }

        let payload = read.read_boxed_slice(length)?;

        Ok(Self {
            key,
            payload: Some(payload),
        })
    }
}
