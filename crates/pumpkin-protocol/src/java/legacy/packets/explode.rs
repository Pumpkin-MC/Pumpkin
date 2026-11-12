//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/explode.rs`.

use crate::java::client::play::CExplosion;
use crate::java::legacy::LegacyWrite;
use crate::ser::NetworkWriteExt;
use crate::{IdOr, codec::var_int::VarInt};
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CExplosion {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_1_19_3 {
            write.write_f64_be(self.center.x)?;
            write.write_f64_be(self.center.y)?;
            write.write_f64_be(self.center.z)?;
        } else {
            write.write_f32_be(self.center.x as f32)?;
            write.write_f32_be(self.center.y as f32)?;
            write.write_f32_be(self.center.z as f32)?;
        }

        if *version >= JavaMinecraftVersion::V_1_21_2 {
            if *version >= JavaMinecraftVersion::V_1_21_9 {
                write.write_f32_be(self.radius)?;
                write.write_i32_be(self.block_count)?;
            }

            write.write_option(&self.knockback, |w, k| {
                w.write_f64_be(k.x)?;
                w.write_f64_be(k.y)?;
                w.write_f64_be(k.z)?;
                Ok(())
            })?;

            write.write_var_int(&self.particle)?;

            crate::IdOr::<crate::SoundEvent>::write(&self.sound, &mut write, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
            })?;

            if *version >= JavaMinecraftVersion::V_1_21_9 {
                write.write_var_int(&self.block_particles_pool_size)?;
            }

            // Whether the explosion sound is played, added in 26.3
            if *version >= JavaMinecraftVersion::V_26_3 {
                write.write_bool(true)?;
            }
        } else {
            write.write_f32_be(self.radius)?;

            if *version >= JavaMinecraftVersion::V_1_17 {
                write.write_var_int(&VarInt(0))?;
            } else {
                write.write_i32_be(0)?;
            }

            if let Some(knockback) = self.knockback {
                write.write_f32_be(knockback.x as f32)?;
                write.write_f32_be(knockback.y as f32)?;
                write.write_f32_be(knockback.z as f32)?;
            } else {
                write.write_f32_be(0.0)?;
                write.write_f32_be(0.0)?;
                write.write_f32_be(0.0)?;
            }

            if *version >= JavaMinecraftVersion::V_1_20_3 {
                // Block interaction: 1 = DESTROY_BLOCKS
                write.write_var_int(&VarInt(1))?;

                let small_particle = VarInt(pumpkin_data::particle::Particle::Explosion as i32);
                write.write_var_int(&small_particle)?;

                write.write_var_int(&self.particle)?;

                if *version >= JavaMinecraftVersion::V_1_20_5 {
                    crate::IdOr::<crate::SoundEvent>::write(&self.sound, &mut write, |w, e| {
                        w.write_string(&e.sound_name)?;
                        w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
                    })?;
                } else {
                    let (sound_name, range) = match &self.sound {
                        IdOr::Id(id) => {
                            let name = pumpkin_data::sound::Sound::NAMES
                                .get(*id as usize)
                                .copied()
                                .unwrap_or("minecraft:entity.generic.explode");
                            (name, None)
                        }
                        IdOr::Value(event) => (event.sound_name.as_str(), event.range),
                    };
                    write.write_string(sound_name)?;
                    write.write_option(&range, |w, r| w.write_f32_be(*r))?;
                }
            }

            // Block count: 0
            write.write_var_int(&VarInt(0))?;
        }

        Ok(())
    }
}
