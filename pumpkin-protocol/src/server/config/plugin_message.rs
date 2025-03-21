use std::io::Read;

use pumpkin_data::packet::serverbound::CONFIG_CUSTOM_PAYLOAD;
use pumpkin_macros::packet;

use crate::{
    ServerPacket,
    codec::identifier::Identifier,
    ser::{NetworkReadExt, ReadingError},
};
const MAX_PAYLOAD_SIZE: usize = 1048576;

#[packet(CONFIG_CUSTOM_PAYLOAD)]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: Box<[u8]>,
}

impl ServerPacket for SPluginMessage {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            channel: read.get_identifier()?,
            data: read.read_remaining_to_boxed_slice(MAX_PAYLOAD_SIZE)?,
        })
    }
}
