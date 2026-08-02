use pumpkin_data::packet::serverbound::PLAY_COOKIE_RESPONSE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[java_packet(PLAY_COOKIE_RESPONSE)]
/// Response to a `CCookieRequest` (play) from the server.
/// The Notchian (vanilla) server only accepts responses of up to 5 KiB in size.
pub struct SCookieResponse<'a> {
    pub key: &'a str,
    pub payload: Option<&'a [u8]>, // 5120,
}

const MAX_COOKIE_LENGTH: usize = 5120;

impl<'a> ServerPacket<'a> for SCookieResponse<'a> {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let key = read.get_str_borrowed()?;
        let has_payload = read.get_bool()?;

        if !has_payload {
            return Ok(Self { key, payload: None });
        }

        let payload_length = read.get_var_int()?.0 as usize;
        if payload_length > MAX_COOKIE_LENGTH {
            return Err(ReadingError::TooLarge("SCookieResponse".to_string()));
        }

        let payload = read.read_slice_borrowed(payload_length)?;

        Ok(Self {
            key,
            payload: Some(payload),
        })
    }
}
