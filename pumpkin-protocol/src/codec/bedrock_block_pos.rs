use std::io::{Error, Write};

use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

/// A wrapper for `BlockPos` that handles Bedrock-specific network serialization.
///
/// Bedrock Edition encodes coordinates differently than Java Edition, using
/// `VarInt`'s to save bandwidth.
pub struct NetworkPos(pub BlockPos);

impl NetworkPos {
    /// Writes coordinates where all axes (X, Y, Z) are treated as signed `VarInt`.
    pub fn write_signed<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.0.0.x).write(writer)?;
        VarInt(self.0.0.y).write(writer)?;
        VarInt(self.0.0.z).write(writer)
    }
}

impl PacketWrite for NetworkPos {
    /// The default Bedrock network encoding for block positions.
    ///
    /// Note: X and Z are signed (`VarInt`), but Y is unsigned (`VarUInt`).
    /// This matches the standard Bedrock block action and block update packets
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.0.0.x).write(writer)?;
        VarUInt(self.0.0.y as u32).write(writer)?;
        VarInt(self.0.0.z).write(writer)
    }
}
