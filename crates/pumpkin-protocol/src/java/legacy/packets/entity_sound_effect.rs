//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/entity_sound_effect.rs`.

use crate::java::client::play::CEntitySoundEffect;
use crate::java::legacy::LegacyWrite;
use crate::{IdOr, VarInt, ser::NetworkWriteExt, ser::WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CEntitySoundEffect {
    fn write_legacy(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_19_3 {
            crate::IdOr::<crate::SoundEvent>::write(&self.sound_event, &mut write, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32(*r))
            })?;
        } else {
            let sound_id = match &self.sound_event {
                IdOr::Id(id) => *id,
                IdOr::Value(_) => 0,
            };
            write.write_var_int(&VarInt(i32::from(sound_id)))?;
        }

        write.write_var_int(&self.sound_category)?;
        write.write_var_int(&self.entity_id)?;
        write.write_f32(self.volume)?;

        if *version >= JavaMinecraftVersion::V_1_10 {
            write.write_f32(self.pitch)?;
        } else {
            let pitch_byte = (self.pitch * 63.0).round().clamp(0.0, 255.0) as u8;
            write.write_u8(pitch_byte)?;
        }

        if *version >= JavaMinecraftVersion::V_1_19 {
            write.write_i64(self.seed)?;
        }

        Ok(())
    }
}
