use pumpkin_data::packet::clientbound::PLAY_EXPLODE;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use crate::{IdOr, SoundEvent, codec::var_int::VarInt};
use pumpkin_util::version::JavaMinecraftVersion;

use super::particle::particle_id_for_version;

/// Notifies the client that an explosion has occurred.
///
/// This is a high-level packet that handles the visual, auditory, and physical
/// effects of an explosion in a single call. It triggers the explosion particles,
/// plays the sound at the source, and applies knockback to the player.
#[java_packet(PLAY_EXPLODE)]
pub struct CExplosion {
    /// The center coordinates of the explosion.
    pub center: Vector3<f64>,
    /// The strength/radius of the explosion.
    /// Higher values increase the visual size of the particle effect.
    pub radius: f32,
    /// The number of blocks affected/destroyed.
    /// Note: The actual block list is typically handled via a separate packet or
    /// following byte array in older versions, but this field specifies the count.
    pub block_count: i32,
    /// The impulse/knockback applied to the player receiving this packet.
    /// If None, no velocity change is applied.
    pub knockback: Option<Vector3<f64>>,
    /// The ID of the particle to use for the explosion (e.g., `minecraft:explosion`).
    pub particle: VarInt,
    /// The sound to play (e.g., `minecraft:entity.generic.explode`).
    pub sound: IdOr<SoundEvent>,
    /// The size of the block particles pool, used for debris visuals.
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
        let particle = particle_id_for_version(self.particle, *version);
        write.write_var_int(&particle)?;
        match &self.sound {
            IdOr::Id(id) => write.write_var_int(&VarInt((*id + 1) as i32))?,
            IdOr::Value(event) => {
                write.write_var_int(&VarInt(0))?;
                write.write_string(&event.sound_name)?;
                write.write_option(&event.range, |w, r| w.write_f32_be(*r))?;
            }
        }
        write.write_var_int(&self.block_particles_pool_size)?;
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
