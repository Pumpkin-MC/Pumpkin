use std::io::{Error, Write};

use pumpkin_util::BedrockVersion;
use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

/// A wrapper for `BlockPos` that handles Bedrock-specific network serialization.
///
/// Bedrock Edition encodes coordinates differently than Java Edition, using
/// `VarInt`'s to save bandwidth. The Y-axis encoding changed in protocol version
/// `BEDROCK_VERSION_1_26_10` (944): earlier versions use unsigned `VarUInt` for Y,
/// while 944+ uses signed `VarInt`.
pub struct NetworkPos {
    pub pos: BlockPos,
    pub version: BedrockVersion,
}

impl NetworkPos {
    /// Creates a `NetworkPos` with default encoding for the given protocol version.
    #[must_use]
    pub const fn for_protocol(pos: BlockPos, version: BedrockVersion) -> Self {
        Self { pos, version }
    }
}

impl PacketWrite for NetworkPos {
    /// Encodes block position with protocol-aware Y-axis handling.
    ///
    /// X and Z are always signed `VarInt`. Y encoding depends on version:
    /// - protocol < 944: Y is unsigned `VarUInt`
    /// - protocol >= 944: Y is signed `VarInt`
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.pos.0.x).write(writer)?;
        if self.version >= BedrockVersion::V1_26_10 {
            VarInt(self.pos.0.y).write(writer)?;
        } else {
            VarUInt(self.pos.0.y as u32).write(writer)?;
        }
        VarInt(self.pos.0.z).write(writer)
    }
}
