use pumpkin_data::packet::clientbound::PLAY_TAG_QUERY;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_TAG_QUERY)]
pub struct CTagQueryResponse<'a> {
    pub transaction_id: VarInt,
    pub nbt_bytes: &'a [u8],
}

impl<'a> CTagQueryResponse<'a> {
    #[must_use]
    pub const fn new(transaction_id: VarInt, nbt_bytes: &'a [u8]) -> Self {
        Self {
            transaction_id,
            nbt_bytes,
        }
    }
}

impl ClientPacket for CTagQueryResponse<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.transaction_id)?;
        write.write_slice(self.nbt_bytes)?;
        Ok(())
    }
}
