//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/player_position.rs`.

use crate::java::client::play::CPlayerPosition;
use crate::java::legacy::LegacyWrite;
use crate::{PositionFlag, ser::NetworkWriteExt, ser::WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CPlayerPosition {
    fn write_legacy(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            // Reordered and added delta/int flags in 1.21.2
            write.write_var_int(&self.teleport_id)?;
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
            write.write_f64_be(self.delta.x)?;
            write.write_f64_be(self.delta.y)?;
            write.write_f64_be(self.delta.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
            write.write_i32_be(PositionFlag::get_bitfield(self.relatives.as_slice()))?;
        } else {
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
            if version >= &JavaMinecraftVersion::V_1_8 {
                // Relative flags added in 1.8
                write.write_u8(PositionFlag::get_bitfield(self.relatives.as_slice()) as u8)?;
            } else {
                // 1.7.x: on_ground boolean
                write.write_bool(false)?;
            }
            if version >= &JavaMinecraftVersion::V_1_9 {
                // Teleport confirmation ID added in 1.9
                write.write_var_int(&self.teleport_id)?;
            }
            if *version >= JavaMinecraftVersion::V_1_17
                && *version <= JavaMinecraftVersion::V_1_19_3
            {
                write.write_bool(false)?;
            }
        }
        Ok(())
    }
}
