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
    /// Creates a new `CDeleteChat` packet using a raw signature ID `VarInt`.
    #[must_use]
    pub const fn from_id(signature_id: VarInt) -> Self {
        Self {
            signature_id,
            signature: None,
        }
    }

    /// Creates a new `CDeleteChat` packet using a 0-indexed signature cache ID.
    ///
    /// In Minecraft's `MessageSignature.Packed`, cached signatures are written as `cache_id + 1`.
    #[must_use]
    pub const fn from_cache_id(cache_id: i32) -> Self {
        Self {
            signature_id: VarInt(cache_id + 1),
            signature: None,
        }
    }

    /// Creates a new `CDeleteChat` packet using a full 256-byte message signature.
    ///
    /// In Minecraft's `MessageSignature.Packed`, a full signature is written as `VarInt(0)`
    /// followed by the 256 raw signature bytes.
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
