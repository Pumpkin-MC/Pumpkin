//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/entity_velocity.rs`.

use crate::java::client::play::CEntityVelocity;
use crate::java::legacy::LegacyWrite;
use crate::{ser::NetworkWriteExt, ser::WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CEntityVelocity {
    fn write_legacy(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }

        // Protocol 773+ uses packed velocity; 772 and below use three i16 components.
        if version >= &JavaMinecraftVersion::V_1_21_9 {
            self.velocity.write(&mut write)?;
        } else {
            crate::java::legacy::write_legacy_velocity(&mut write, &self.velocity.0)?;
        }

        Ok(())
    }
}
