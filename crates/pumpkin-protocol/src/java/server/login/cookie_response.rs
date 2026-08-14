use pumpkin_data::packet::serverbound::LOGIN_COOKIE_RESPONSE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[java_packet(LOGIN_COOKIE_RESPONSE)]
/// Response to a `CCookieRequest` (login) from the server.
/// The Notchian server only accepts responses of up to 5 kiB in size.
pub struct SLoginCookieResponse<'a> {
    pub key: &'a str,
    pub payload: Option<&'a [u8]>, // 5120,
}

const MAX_COOKIE_LENGTH: usize = 5120;

impl<'a> ServerPacket<'a> for SLoginCookieResponse<'a> {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let key = read.get_str_borrowed()?;
        let has_payload = read.get_bool()?;

        if !has_payload {
            return Ok(Self { key, payload: None });
        }

        let payload_length = read.get_var_int()?;
        let length = payload_length.0 as usize;
        if length > MAX_COOKIE_LENGTH {
            return Err(ReadingError::TooLarge("SLoginCookieResponse".to_string()));
        }

        let payload = read.read_slice_borrowed(length)?;

        Ok(Self {
            key,
            payload: Some(payload),
        })
    }
}
