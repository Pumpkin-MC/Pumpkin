use bytes::Buf;
use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket,
};

#[server_packet(LOGIN_HELLO)]
pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl ServerPacket for SLoginStart {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            name: bytebuf.try_get_string_len(16)?,
            uuid: bytebuf.try_get_uuid()?,
        })
    }
}
