use crate::bytebuf::ByteBufMut;
use bytes::Buf;
use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::packet;

use crate::{
    ClientPacket, ServerPacket,
    bytebuf::{ByteBuf, ReadingError},
};

#[packet(LOGIN_HELLO)]
pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl ClientPacket for SLoginStart {
    fn write(&self, bytebuf: &mut impl bytes::BufMut) {
        bytebuf.put_string_len(&self.name, 16);
        bytebuf.put_uuid(&self.uuid);
    }
}

impl ServerPacket for SLoginStart {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            name: bytebuf.try_get_string_len(16)?,
            uuid: bytebuf.try_get_uuid()?,
        })
    }
}
