//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/player_spawn_position.rs`.

use crate::java::client::play::CPlayerSpawnPosition;
use crate::java::legacy::{LegacyWrite, LegacyWriteExt};
use crate::ser::{NetworkWriteExt, WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CPlayerSpawnPosition {
    fn write_legacy(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_21_9 {
            write.write_string(&self.dimension_name)?;
        }

        if *version >= JavaMinecraftVersion::V_1_8 {
            write.write_block_pos_legacy(&self.location, version)?;
        } else {
            write.write_i32_be(self.location.0.x)?;
            write.write_i32_be(self.location.0.y)?;
            write.write_i32_be(self.location.0.z)?;
        }

        if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_f32_be(self.yaw)?;
        }

        if *version >= JavaMinecraftVersion::V_1_21_9 {
            write.write_f32_be(self.pitch)?;
        }

        Ok(())
    }
}
