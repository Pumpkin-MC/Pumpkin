use crate::{ReadingError, ServerPacket, VarInt, ser::NetworkReadExt};
use pumpkin_data::packet::serverbound::LOGIN_CUSTOM_QUERY_ANSWER;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

const MAX_PAYLOAD_SIZE: usize = 1_048_576;

#[java_packet(LOGIN_CUSTOM_QUERY_ANSWER)]
pub struct SLoginPluginResponse {
    pub message_id: VarInt,
    pub data: Option<Box<[u8]>>,
}

impl<'a> ServerPacket<'a> for SLoginPluginResponse {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            message_id: read.get_var_int()?,
            data: read.get_option(|v| crate::ser::read_remaining_bytes(v, MAX_PAYLOAD_SIZE))?,
        })
    }
}
