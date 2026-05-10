use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_LEVEL_PARTICLES;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// Spawns a cluster of particles at a specific location.
///
/// This is the most versatile visual packet in the protocol. It allows for
/// precise control over particle density, spread, and speed. It can also
/// carry extra data for complex particles like redstone dust (color) or
/// block/item breaking (textures).
#[java_packet(PLAY_LEVEL_PARTICLES)]
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
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_bool(self.force_spawn)?;
        write.write_bool(self.important)?;

        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;

        write.write_f32_be(self.offset.x)?;
        write.write_f32_be(self.offset.y)?;
        write.write_f32_be(self.offset.z)?;

        write.write_f32_be(self.max_speed)?;
        write.write_i32_be(self.particle_count)?;
        write.write_var_int(&self.particle_id)?;

        write.write_all(self.data).map_err(WritingError::IoError)
    }
}
