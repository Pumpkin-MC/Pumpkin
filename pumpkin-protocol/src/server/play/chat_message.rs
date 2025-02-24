use bytes::{Buf, Bytes};
use pumpkin_data::packet::serverbound::PLAY_CHAT;
use pumpkin_macros::packet;

use crate::{
    FixedBitSet, ServerPacket, VarInt,
    bytebuf::{ByteBuf, ReadingError},
};

// derive(Deserialize)]
#[packet(PLAY_CHAT)]
pub struct SChatMessage {
    pub message: String,
    pub timestamp: i64,
    pub salt: i64,
    pub signature: Option<Bytes>,
    pub message_count: VarInt,
    pub acknowledged: FixedBitSet,
}

// TODO
impl ServerPacket for SChatMessage {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            message: bytebuf.try_get_string()?,
            timestamp: bytebuf.try_get_i64()?,
            salt: bytebuf.try_get_i64()?,
            signature: bytebuf.try_get_option(|v| v.try_copy_to_bytes(256))?,
            message_count: bytebuf.try_get_var_int()?,
            acknowledged: bytebuf.try_get_fixed_bitset(20)?,
        })
    }
}
