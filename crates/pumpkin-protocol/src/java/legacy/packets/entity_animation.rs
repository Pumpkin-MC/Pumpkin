//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/entity_animation.rs`.

use crate::VarInt;
use crate::java::client::play::Animation;
use crate::java::client::play::{CEntityAnimation, CSwingArm};
use crate::java::legacy::LegacyWrite;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CEntityAnimation {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        // 26.3 moved the swings into their own packet and renumbered the remaining animations
        let animation = if *version >= JavaMinecraftVersion::V_26_3 {
            match self.animation {
                2 => 0, // Leave bed
                4 => 1, // Critical effect
                5 => 2, // Magic critical effect
                other => other,
            }
        } else {
            self.animation
        };
        write.write_u8(animation)?;
        Ok(())
    }
}

impl LegacyWrite for CSwingArm {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        if *version >= JavaMinecraftVersion::V_26_3 {
            write.write_var_int(&VarInt(i32::from(self.off_hand)))?;
            // The swing animation of the held item, whacking for 6 ticks is the default one
            write.write_var_int(&VarInt(1))?;
            write.write_var_int(&VarInt(6))?;
        } else if self.off_hand {
            write.write_u8(Animation::SwingOffhand as u8)?;
        } else {
            write.write_u8(Animation::SwingMainArm as u8)?;
        }
        Ok(())
    }
}
