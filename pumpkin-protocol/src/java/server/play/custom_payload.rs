use std::io::Read;

use crate::ServerPacket;
use crate::ser::{NetworkReadExt, ReadingError};
use pumpkin_data::packet::serverbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

#[packet(PLAY_CUSTOM_PAYLOAD)]
pub struct SCustomPayload {
    pub channel: ResourceLocation,
    pub data: Box<[u8]>,
}
impl ServerPacket for SCustomPayload {
    fn read(mut read: impl Read) -> Result<Self, ReadingError> {
        let channel = read.get_resource_location()?;
        let mut data_vec = Vec::new();
        read.read_to_end(&mut data_vec)
            .map_err(|e| ReadingError::Message(e.to_string()))?;
        let data = data_vec.into_boxed_slice();
        Ok(Self { channel, data })
    }
}
