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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f64_be(self.center.x)?;
        write.write_f64_be(self.center.y)?;
        write.write_f64_be(self.center.z)?;

        write.write_f32_be(self.radius)?;
        write.write_i32_be(self.block_count)?;

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

        write.write_var_int(&self.block_particles_pool_size)?;

        // Whether the explosion sound is played, added in 26.3
        write.write_bool(true)?;

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
    fn explosion_particle_id_stays_latest_for_26_3() {
        assert_eq!(
            encoded_particle_id(pumpkin_data::packet::CURRENT_MC_VERSION),
            VarInt(29)
        );
    }
}
