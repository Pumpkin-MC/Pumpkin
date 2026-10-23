//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/entity_position_sync.rs`.

use crate::VarInt;
use crate::java::client::play::CEntityPositionSync;
use crate::java::legacy::LegacyWrite;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CEntityPositionSync {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        // Since 26.3 the position is a path. 0 is a linear path, which is just the end position.
        if version >= &JavaMinecraftVersion::V_26_3 {
            write.write_var_int(&VarInt(0))?;
        }
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            // The delta was replaced by the path in 26.3.
            if version < &JavaMinecraftVersion::V_26_3 {
                write.write_f64_be(self.delta.x)?;
                write.write_f64_be(self.delta.y)?;
                write.write_f64_be(self.delta.z)?;
            }
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
        } else {
            write.write_u8((self.yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8)?;
            write.write_u8((self.pitch.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8)?;
        }
        write.write_bool(self.on_ground)?;
        Ok(())
    }
}
