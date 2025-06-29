use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_CHAT;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[derive(Serialize)]
#[packet(PLAY_CHAT)]
pub struct SChatMessage {
    pub message: String,
    pub timestamp: i64,
    pub salt: i64,
    pub signature: Option<Box<[u8]>>,
    pub message_count: VarInt,
    pub acknowledged: Box<[u8]>, // Bitset fixed 20 bits
    pub checksum: u8,            // 1.21.5 "fingerprint" checksum
}

impl ServerPacket for SChatMessage {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        Ok(Self {
            message: read.get_string_bounded(256)?,
            timestamp: read.get_i64_be()?,
            salt: read.get_i64_be()?,
            signature: read.get_option(|v| v.read_boxed_slice(256))?,
            message_count: read.get_var_int()?,
            acknowledged: read.get_fixed_bitset(20)?,
            checksum: read.get_u8_be()?,
        })
    }
}
