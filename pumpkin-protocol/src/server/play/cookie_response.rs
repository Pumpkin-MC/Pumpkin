use crate::{
    ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
    codec::identifier::Identifier,
};
use bytes::Buf;
use pumpkin_data::packet::serverbound::PLAY_COOKIE_RESPONSE;
use pumpkin_macros::packet;

#[packet(PLAY_COOKIE_RESPONSE)]
/// Response to a Cookie Request (play) from the server.
/// The Notchian (vanilla) server only accepts responses of up to 5 kiB in size.
pub struct SCookieResponse {
    pub key: Identifier,
    pub has_payload: bool,
    pub payload_length: Option<VarInt>,
    pub payload: Option<bytes::Bytes>, // 5120,
}

const MAX_COOKIE_LENGTH: usize = 5120;

impl ServerPacket for SCookieResponse {
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
