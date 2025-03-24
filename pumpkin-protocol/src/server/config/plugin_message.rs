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
pub struct SPluginMessage<'a> {
    pub channel: Identifier<'a>,
    pub data: Box<[u8]>,
}

impl ServerPacket for SPluginMessage<'_> {
    fn read(mut read: impl Read) -> Result<Self, ReadingError> {
        Ok(Self {
            channel: read.get_identifier()?,
            data: read.read_remaining_to_boxed_slice(MAX_PAYLOAD_SIZE)?,
        })
    }
}
