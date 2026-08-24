use pumpkin_data::{
    packet::clientbound::play::EXPLODE, sound_id_remap::remap_sound_id_for_version,
};
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, IdOr, SoundEvent, codec::var_int::VarInt};

use super::particle::particle_id_for_version;

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

            let particle = particle_id_for_version(self.particle, *version);
            write.write_var_int(&particle)?;

            let sound_event = match &self.sound {
                IdOr::Id(id) => IdOr::Id(remap_sound_id_for_version(*id, *version)),
                IdOr::Value(value) => IdOr::Value(value.clone()),
            };
            crate::IdOr::<crate::SoundEvent>::write(&sound_event, &mut write, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
            })?;

            if *version >= JavaMinecraftVersion::V_1_21_9 {
                write.write_var_int(&self.block_particles_pool_size)?;
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

                let small_particle = particle_id_for_version(
                    VarInt(pumpkin_data::particle::Particle::Explosion as i32),
                    *version,
                );
                write.write_var_int(&small_particle)?;

                let particle = particle_id_for_version(self.particle, *version);
                write.write_var_int(&particle)?;

                if *version >= JavaMinecraftVersion::V_1_20_5 {
                    let sound_event = match &self.sound {
                        IdOr::Id(id) => IdOr::Id(remap_sound_id_for_version(*id, *version)),
                        IdOr::Value(value) => IdOr::Value(value.clone()),
                    };
                    crate::IdOr::<crate::SoundEvent>::write(&sound_event, &mut write, |w, e| {
                        w.write_string(&e.sound_name)?;
                        w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
                    })?;
                } else {
                    let (sound_name, range) = match &self.sound {
                        IdOr::Id(id) => {
                            let remapped = remap_sound_id_for_version(*id, *version);
                            let name = pumpkin_data::sound::Sound::NAMES
                                .get(remapped as usize)
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

    #[test]
    fn explosion_particle_id_remaps_for_1_21_11() {
        assert_eq!(
            encoded_particle_id(JavaMinecraftVersion::V_1_21_11),
            VarInt(22)
        );
    }

    #[test]
    fn explosion_particle_id_stays_latest_for_26_2() {
        assert_eq!(
            encoded_particle_id(JavaMinecraftVersion::V_26_2),
            VarInt(29)
        );
    }
}
