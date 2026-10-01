//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/player_rotation.rs`.

use crate::java::client::play::CPlayerRotation;
use crate::java::legacy::LegacyWrite;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CPlayerRotation {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32_be(self.yaw)?;
        if *version >= JavaMinecraftVersion::V_1_21_9 {
            write.write_bool(false)?;
        }
        write.write_f32_be(self.pitch)?;
        if *version >= JavaMinecraftVersion::V_1_21_9 {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
