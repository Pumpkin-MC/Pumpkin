use pumpkin_data::packet::clientbound::play::EXPLODE;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, IdOr, SoundEvent, codec::var_int::VarInt};

/// Notifies the client that an explosion has occurred.
///
/// This is a high-level packet that handles the visual, auditory, and physical
/// effects of an explosion in a single call. It triggers the explosion particles,
/// plays the sound at the source, and applies knockback to the player.
#[java_packet(EXPLODE)]
#[derive(Clone, PartialEq)]
pub struct CExplosion {
    /// The center coordinates of the explosion.
    pub center: Vector3<f64>,
    /// The strength/radius of the explosion.
    /// Higher values increase the visual size of the particle effect.
    pub radius: f32,
    /// The number of blocks affected/destroyed.
    pub block_count: i32,
    /// The impulse/knockback applied to the player receiving this packet.
    /// If None, no velocity change is applied.
    pub knockback: Option<Vector3<f64>>,
    /// The ID of the particle to use for the explosion (e.g., `minecraft:explosion_emitter`).
    pub particle: VarInt,
    /// The sound to play (e.g., `minecraft:entity.generic.explode`).
    pub sound: IdOr<SoundEvent>,
    /// The size of the block particles pool, used for debris visuals in 1.21.9+.
    pub block_particles_pool_size: VarInt,
}

impl CExplosion {
    #[must_use]
    pub const fn new(
        center: Vector3<f64>,
        radius: f32,
        block_count: i32,
        knockback: Option<Vector3<f64>>,
        particle: VarInt,
        sound: IdOr<SoundEvent>,
    ) -> Self {
        Self {
            center,
            radius,
            block_count,
            knockback,
            particle,
            sound,
            block_particles_pool_size: VarInt(0),
        }
    }
}

impl ClientPacket for CExplosion {
    fn write_packet_data(
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
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use pumpkin_data::particle::Particle;
    use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

    use crate::{ClientPacket, IdOr, VarInt};

    use super::CExplosion;

    fn encoded_particle_id(version: JavaMinecraftVersion) -> VarInt {
        let packet = CExplosion::new(
            Vector3::new(0.0, 0.0, 0.0),
            4.0,
            0,
            None,
            VarInt(Particle::ExplosionEmitter as i32),
            IdOr::Id(0),
        );
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version).unwrap();

        let mut cursor = Cursor::new(bytes);
        cursor.seek(SeekFrom::Start(33)).unwrap();
        VarInt::decode(&mut cursor).unwrap()
    }

    fn layout_len(version: JavaMinecraftVersion) -> usize {
        let packet = CExplosion::new(
            Vector3::new(0.0, 0.0, 0.0),
            4.0,
            0,
            Some(Vector3::new(1.0, 2.0, 3.0)),
            VarInt(Particle::Explosion as i32),
            IdOr::Id(0),
        );
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version).unwrap();
        bytes.len()
    }

    #[test]
    fn explosion_layout_for_1_21_5() {
        // center, optional knockback (present), particle id, sound id
        assert_eq!(
            layout_len(JavaMinecraftVersion::V_1_21_5),
            24 + 1 + 24 + 1 + 1
        );
    }

    #[test]
    fn explosion_layout_for_1_21() {
        // center, radius, record count, knockback, block interaction,
        // small particle, particle, sound id
        assert_eq!(
            layout_len(JavaMinecraftVersion::V_1_21),
            24 + 4 + 1 + 12 + 1 + 1 + 1 + 1
        );
    }

    #[test]
    fn explosion_particle_id_stays_latest_for_26_3() {
        assert_eq!(
            encoded_particle_id(JavaMinecraftVersion::V_26_3),
            VarInt(29)
        );
    }
}
