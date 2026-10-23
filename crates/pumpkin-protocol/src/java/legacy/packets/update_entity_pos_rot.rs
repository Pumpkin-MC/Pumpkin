//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/update_entity_pos_rot.rs`.

use crate::java::client::play::CUpdateEntityPosRot;
use crate::java::legacy::LegacyWrite;
use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CUpdateEntityPosRot {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }
        // Since 26.3 the on ground flag and the delta step count are packed into a var int in
        // front of the delta, 0 steps being a single linear delta.
        if *version >= JavaMinecraftVersion::V_26_3 {
            write.write_var_int(&VarInt(i32::from(self.on_ground)))?;
        }
        if *version >= JavaMinecraftVersion::V_1_9 {
            write.write_i16_be(self.delta.x)?;
            write.write_i16_be(self.delta.y)?;
            write.write_i16_be(self.delta.z)?;
        } else {
            write.write_i8((self.delta.x / 128) as i8)?;
            write.write_i8((self.delta.y / 128) as i8)?;
            write.write_i8((self.delta.z / 128) as i8)?;
        }
        write.write_u8(self.yaw)?;
        write.write_u8(self.pitch)?;
        if *version >= JavaMinecraftVersion::V_1_8 && *version < JavaMinecraftVersion::V_26_3 {
            write.write_bool(self.on_ground)?;
        }
        Ok(())
    }
}
