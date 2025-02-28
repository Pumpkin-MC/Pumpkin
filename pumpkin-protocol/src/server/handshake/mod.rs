use crate::bytebuf::ByteBufMut;
use crate::{
    ClientPacket, ConnectionState, ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
};
use bytes::Buf;
use pumpkin_data::packet::serverbound::HANDSHAKE_INTENTION;
use pumpkin_macros::packet;

#[packet(HANDSHAKE_INTENTION)]
pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}

impl ClientPacket for SHandShake {
    fn write(&self, bytebuf: &mut impl bytes::BufMut) {
        bytebuf.put_var_int(&self.protocol_version);
        bytebuf.put_string_len(&self.server_address, 255);
        bytebuf.put_u16(self.server_port);
        bytebuf.put_var_int(&VarInt(self.next_state as i32));
    }
}

impl ServerPacket for SHandShake {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            protocol_version: bytebuf.try_get_var_int()?,
            server_address: bytebuf.try_get_string_len(255)?,
            server_port: bytebuf.try_get_u16()?,
            next_state: bytebuf
                .try_get_var_int()?
                .try_into()
                .map_err(|_| ReadingError::Message("Invalid Status".to_string()))?,
        })
    }
}
