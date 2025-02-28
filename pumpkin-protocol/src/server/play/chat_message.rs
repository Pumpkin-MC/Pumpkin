use bytes::Buf;
use pumpkin_data::packet::serverbound::PLAY_CHAT;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{
    ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
};

#[derive(Serialize)]
#[packet(PLAY_CHAT)]
pub struct SChatMessage {
    pub message: String,
    pub timestamp: i64,
    pub salt: i64,
    pub signature: Option<Vec<u8>>,
    pub message_count: VarInt,
    pub acknowledged: Vec<u8>,
}

impl SChatMessage {
    pub fn new(
        message: String,
        timestamp: i64,
        salt: i64,
        signature: Option<Vec<u8>>,
        message_count: VarInt,
        acknowledged: Vec<u8>,
    ) -> Self {
        Self {
            message,
            timestamp,
            salt,
            signature,
            message_count,
            acknowledged,
        }
    }
}

// TODO
impl ServerPacket for SChatMessage {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            message: bytebuf.try_get_string_len(256)?,
            timestamp: bytebuf.try_get_i64()?,
            salt: bytebuf.try_get_i64()?,
            signature: bytebuf.try_get_option(|v| v.try_copy_to_bytes(256).map(|s| s.to_vec()))?,
            message_count: bytebuf.try_get_var_int()?,
            acknowledged: bytebuf.try_get_fixed_bitset(20).map(|s| s.to_vec())?,
        })
    }
}
