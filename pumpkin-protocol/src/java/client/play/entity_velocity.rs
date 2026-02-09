use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_MOTION;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{
    ClientPacket, VarInt, WritingError,
    codec::velocity::Velocity,
    ser::NetworkWriteExt,
};

/// Updates the velocity of an entity.
///
/// This packet informs the client of a sudden change in an entity's movement,
/// such as knockback from an attack, explosions, or being launched by a piston.
#[java_packet(PLAY_SET_ENTITY_MOTION)]
pub struct CEntityVelocity {
    /// The Entity ID of the entity whose velocity is being set
    pub entity_id: VarInt,
    /// The velocity vector
    pub velocity: Velocity,
}

impl CEntityVelocity {
    #[must_use]
    pub const fn new(entity_id: VarInt, velocity: Vector3<f64>) -> Self {
        Self {
            entity_id,
            velocity: Velocity(velocity),
        }
    }
}

impl ClientPacket for CEntityVelocity {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;

        // Protocol 773+ uses packed velocity; 772 and below use three i16 components.
        if version >= &MinecraftVersion::V_1_21_9 {
            self.velocity.write(&mut write)?;
        } else {
            write.write_i16_be(self.velocity.0.x as i16)?;
            write.write_i16_be(self.velocity.0.y as i16)?;
            write.write_i16_be(self.velocity.0.z as i16)?;
        }

        Ok(())
    }
}
