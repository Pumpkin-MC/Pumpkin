use pumpkin_data::packet::clientbound::play::DELETE_CHAT;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(DELETE_CHAT)]
pub struct CDeleteChat<'a> {
    pub signature_id: VarInt,
    pub signature: Option<&'a [u8]>,
}

impl<'a> CDeleteChat<'a> {
    #[must_use]
    pub const fn from_id(signature_id: VarInt) -> Self {
        Self {
            signature_id,
            signature: None,
        }
    }

    #[must_use]
    pub const fn from_signature(signature: &'a [u8]) -> Self {
        Self {
            signature_id: VarInt(0),
            signature: Some(signature),
        }
    }
}

impl ClientPacket for CDeleteChat<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.signature_id)?;
        if let Some(signature) = self.signature {
            write.write_slice(signature)?;
        }
        Ok(())
    }
}
