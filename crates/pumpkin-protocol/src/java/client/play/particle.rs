use std::io::Write;

use pumpkin_data::packet::clientbound::play::LEVEL_PARTICLES;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, ServerPacket, VarInt,
    codec::particle::ParticleOptionsLayout,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

/// Spawns a cluster of particles at a specific location.
///
/// This is the most versatile visual packet in the protocol. It allows for
/// precise control over particle density, spread, and speed. It can also
/// carry extra data for complex particles like redstone dust (color) or
/// block/item breaking (textures).
#[java_packet(LEVEL_PARTICLES)]
#[derive(Clone, Debug, PartialEq)]
pub struct CParticle<'a> {
    /// If true, the particle renders even if the client's "Particles"
    /// setting is set to "Minimal".
    pub force_spawn: bool,
    /// If true, the distance at which particles are visible is significantly
    /// increased (from 256 to 65536 blocks). Often used for massive events.
    pub important: bool,
    /// The absolute center position of the particle cluster.
    pub position: Vector3<f64>,
    /// The maximum distance from the center that particles can spawn.
    pub offset: Vector3<f32>,
    /// The velocity or "spread" speed of the particles.
    pub max_speed: f32,
    /// The total number of particles to spawn in this cluster.
    pub particle_count: i32,
    /// The ID of the particle type (e.g., `minecraft:flame`).
    pub particle_id: VarInt,
    /// Extra data required by specific particles (e.g., block states for
    /// `block` particles or RGB values for `dust`).
    pub data: &'a [u8],
}

impl<'a> CParticle<'a> {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        force_spawn: bool,
        important: bool,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle_id: VarInt,
        data: &'a [u8],
    ) -> Self {
        Self {
            force_spawn,
            important,
            position,
            offset,
            max_speed,
            particle_count,
            particle_id,
            data,
        }
    }
}

impl ClientPacket for CParticle<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // The particle moved back to the front of the packet in 26.3
        write.write_var_int(&self.particle_id)?;
        write.write_slice(self.data)?;

        write.write_bool(self.important)?;
        write.write_bool(self.force_spawn)?;

        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;

        write.write_f32_be(self.offset.x)?;
        write.write_f32_be(self.offset.y)?;
        write.write_f32_be(self.offset.z)?;

        write.write_f32_be(self.max_speed)?;
        // Since 26.3 the speed is set per axis and the count is a var int, followed by the
        // randomization type, 0 being the default one.
        write.write_f32_be(self.max_speed)?;
        write.write_f32_be(self.max_speed)?;
        write.write_var_int(&VarInt(self.particle_count))?;
        write.write_var_int(&VarInt(0))?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CParticle<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let particle_id = bytebuf.get_var_int()?;
        let data = ParticleOptionsLayout::read_bytes(bytebuf, particle_id)?;

        let important = bytebuf.get_bool()?;
        let force_spawn = bytebuf.get_bool()?;

        let position = Vector3::new(
            bytebuf.get_f64_be()?,
            bytebuf.get_f64_be()?,
            bytebuf.get_f64_be()?,
        );

        let offset = Vector3::new(
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
        );
        let max_speed = bytebuf.get_f32_be()?;
        bytebuf.get_f32_be()?;
        bytebuf.get_f32_be()?;
        let particle_count = bytebuf.get_var_int()?.0;
        bytebuf.get_var_int()?;

        Ok(Self {
            force_spawn,
            important,
            position,
            offset,
            max_speed,
            particle_count,
            particle_id,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{packet::CURRENT_MC_VERSION, particle::Particle};
    use pumpkin_util::math::vector3::Vector3;

    use crate::{ClientPacket, ServerPacket, VarInt, ser::NetworkWriteExt};

    use super::CParticle;

    fn assert_roundtrip(packet: &CParticle<'_>) {
        let mut buf = Vec::new();
        packet
            .write_packet_data(&mut buf, &CURRENT_MC_VERSION)
            .unwrap();
        let mut slice = buf.as_slice();
        let read = CParticle::read(&mut slice, &CURRENT_MC_VERSION).unwrap();
        assert!(slice.is_empty());
        assert_eq!(&read, packet);
    }

    #[test]
    fn read_matches_26_3_write_layout() {
        let packet = CParticle::new(
            true,
            false,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.1, 0.2, 0.3),
            0.5,
            10,
            VarInt(i32::from(Particle::Flame.to_id())),
            &[],
        );
        assert_roundtrip(&packet);
    }

    #[test]
    fn read_stops_particle_data_before_flags() {
        let mut data = Vec::new();
        data.write_i32_be(0x00FF_0000).unwrap();
        data.write_f32_be(2.0).unwrap();
        let packet = CParticle::new(
            false,
            true,
            Vector3::new(4.0, 5.0, 6.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.25,
            1,
            VarInt(i32::from(Particle::Dust.to_id())),
            &data,
        );
        assert_roundtrip(&packet);
    }

    #[test]
    fn read_item_particle_uses_stack_template() {
        let mut data = Vec::new();
        data.write_var_int(&VarInt(1)).unwrap();
        data.write_var_int(&VarInt(1)).unwrap();
        data.write_var_int(&VarInt(0)).unwrap();
        data.write_var_int(&VarInt(0)).unwrap();
        let packet = CParticle::new(
            false,
            false,
            Vector3::new(0.0, 64.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            1,
            VarInt(i32::from(Particle::Item.to_id())),
            &data,
        );
        assert_roundtrip(&packet);
    }
}
