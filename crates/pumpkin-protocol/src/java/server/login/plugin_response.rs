use crate::{ReadingError, ServerPacket, VarInt, ser::NetworkReadExt};
use pumpkin_data::packet::serverbound::login::CUSTOM_QUERY_ANSWER;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

const MAX_PAYLOAD_SIZE: usize = 1_048_576;

#[java_packet(CUSTOM_QUERY_ANSWER)]
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

impl crate::ClientPacket for SLoginPluginResponse {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.message_id)?;
        if let Some(data) = &self.data {
            write.write_bool(true)?;
            write.write_slice(data)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
