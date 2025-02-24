use bytes::Buf;
use pumpkin_data::packet::serverbound::CONFIG_COOKIE_RESPONSE;
use pumpkin_macros::packet;

use crate::{
    ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
    codec::identifier::Identifier,
};

#[packet(CONFIG_COOKIE_RESPONSE)]
/// Response to a Cookie Request (configuration) from the server.
/// The Notchian (vanilla) server only accepts responses of up to 5 kiB in size.
pub struct SConfigCookieResponse {
    pub key: Identifier,
    pub has_payload: bool,
    pub payload_length: Option<VarInt>,
    pub payload: Option<bytes::Bytes>, // 5120,
}

const MAX_COOKIE_LENGTH: usize = 5120;

impl ServerPacket for SConfigCookieResponse {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        let key = bytebuf.try_get_identifier()?;
        let has_payload = bytebuf.try_get_bool()?;

        if !has_payload {
            return Ok(Self {
                key,
                has_payload,
                payload_length: None,
                payload: None,
            });
        }

        let payload_length = bytebuf.try_get_var_int()?;
        let length = payload_length.0;

        let payload = bytebuf.try_copy_to_bytes_len(length as usize, MAX_COOKIE_LENGTH)?;

        Ok(Self {
            key,
            has_payload,
            payload_length: Some(payload_length),
            payload: Some(payload),
        })
    }
}
