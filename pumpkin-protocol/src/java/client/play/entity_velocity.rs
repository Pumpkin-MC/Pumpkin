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

fn encode_legacy_velocity_component(component: f64) -> i16 {
    // Legacy clients (<= 1.21.8 / protocol 772) encode velocity as clamped component * 8000.
    (component.clamp(-3.9, 3.9) * 8000.0) as i16
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
            write.write_i16_be(encode_legacy_velocity_component(self.velocity.0.x))?;
            write.write_i16_be(encode_legacy_velocity_component(self.velocity.0.y))?;
            write.write_i16_be(encode_legacy_velocity_component(self.velocity.0.z))?;
        }

        Ok(())
    }
}
