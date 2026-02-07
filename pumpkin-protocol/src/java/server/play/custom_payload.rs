use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

const MAX_PAYLOAD_SIZE: usize = 1_048_576;

#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct SCustomPayload {
    pub channel: String,
    pub data: Box<[u8]>,
}

impl ServerPacket for SCustomPayload {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        Ok(Self {
            channel: read.get_string()?,
            data: read.read_remaining_to_boxed_slice(MAX_PAYLOAD_SIZE)?,
        })
    }
}
