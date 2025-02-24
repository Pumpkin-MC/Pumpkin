use bytes::Buf;
use pumpkin_data::packet::serverbound::CONFIG_CUSTOM_PAYLOAD;
use pumpkin_macros::packet;

use crate::{
    ServerPacket,
    bytebuf::{ByteBuf, ReadingError},
    codec::identifier::Identifier,
};
const MAX_PAYLOAD_SIZE: usize = 1048576;

#[packet(CONFIG_CUSTOM_PAYLOAD)]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: bytes::Bytes,
}

impl ServerPacket for SPluginMessage {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            channel: bytebuf.try_get_identifier()?,
            data: bytebuf.try_copy_to_bytes_len(bytebuf.remaining(), MAX_PAYLOAD_SIZE)?,
        })
    }
}
