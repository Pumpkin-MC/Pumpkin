use std::io::Read;

use crate::{ReadingError, ServerPacket, VarInt, ser::NetworkReadExt};
use pumpkin_data::packet::serverbound::LOGIN_CUSTOM_QUERY_ANSWER;
use pumpkin_macros::packet;

const MAX_PAYLOAD_SIZE: usize = 1048576;

#[packet(LOGIN_CUSTOM_QUERY_ANSWER)]
pub struct SLoginPluginResponse {
    pub message_id: VarInt,
    pub data: Option<Box<[u8]>>,
}

impl ServerPacket for SLoginPluginResponse {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        Ok(Self {
            message_id: read.get_var_int()?,
            data: read.get_option(|v| v.read_remaining_to_boxed_slice(MAX_PAYLOAD_SIZE))?,
        })
    }
}
