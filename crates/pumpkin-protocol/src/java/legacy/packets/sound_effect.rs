//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/sound_effect.rs`.

use crate::java::client::play::CSoundEffect;
use crate::java::legacy::LegacyWrite;
use crate::{IdOr, VarInt, ser::NetworkWriteExt, ser::WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CSoundEffect {
    fn write_legacy(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_19_3 {
            crate::IdOr::<crate::SoundEvent>::write(&self.sound_event, &mut write, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
            })?;
        } else if *version >= JavaMinecraftVersion::V_1_9 {
            let sound_id = match &self.sound_event {
                IdOr::Id(id) => *id,
                IdOr::Value(_) => 0,
            };
            write.write_var_int(&VarInt(i32::from(sound_id)))?;
        } else {
            let sound_name: &str = match &self.sound_event {
                IdOr::Id(id) => pumpkin_data::sound::Sound::NAMES
                    .get(usize::from(*id))
                    .copied()
                    .unwrap_or("ambient.cave"),
                IdOr::Value(event) => &event.sound_name,
            };
            write.write_string(sound_name)?;
        }

        if *version >= JavaMinecraftVersion::V_1_9 {
            write.write_var_int(&self.sound_category)?;
        }

        write.write_i32_be(self.position.x)?;
        write.write_i32_be(self.position.y)?;
        write.write_i32_be(self.position.z)?;
        write.write_f32_be(self.volume)?;

        if *version >= JavaMinecraftVersion::V_1_10 {
            write.write_f32_be(self.pitch)?;
        } else {
            let pitch_byte = (self.pitch * 63.0).round().clamp(0.0, 255.0) as u8;
            write.write_u8(pitch_byte)?;
        }

        if *version >= JavaMinecraftVersion::V_1_19 {
            write.write_i64_be(self.seed)?;
        }

        Ok(())
    }
}
