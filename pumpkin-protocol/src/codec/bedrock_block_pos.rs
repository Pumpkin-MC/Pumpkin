use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    ser::NetworkWriteExt,
};

/// Bedrocks Writes and Reads BlockPos types in Packets differently
pub struct BedrockPos(pub BlockPos);

impl serde::Serialize for BedrockPos {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut buf = Vec::new();
        buf.write_var_int(&VarInt(self.0.0.x)).unwrap();
        buf.write_var_uint(&VarUInt(self.0.0.y as u32)).unwrap();
        buf.write_var_int(&VarInt(self.0.0.z)).unwrap();
        serializer.serialize_bytes(&buf)
    }
}
