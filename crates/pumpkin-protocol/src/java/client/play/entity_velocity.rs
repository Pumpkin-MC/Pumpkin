use std::io::Write;

use pumpkin_data::packet::clientbound::play::SET_ENTITY_MOTION;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ClientPacket, VarInt, WritingError, codec::lp_vector_3d::LpVector3d, ser::NetworkWriteExt,
};

/// Updates the velocity of an entity.
///
/// This packet informs the client of a sudden change in an entity's movement,
/// such as knockback from an attack, explosions, or being launched by a piston.
#[java_packet(SET_ENTITY_MOTION)]
pub struct CEntityVelocity {
    /// The Entity ID of the entity whose velocity is being set
    pub entity_id: VarInt,
    /// The velocity vector
    pub velocity: LpVector3d,
}

impl CEntityVelocity {
    #[must_use]
    pub const fn new(entity_id: VarInt, velocity: Vector3<f64>) -> Self {
        Self {
            entity_id,
            velocity: LpVector3d(velocity),
        }
    }
}

impl ClientPacket for CEntityVelocity {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;

        // Protocol 773+ uses packed velocity; 772 and below use three i16 components.
        self.velocity.write(&mut write)?;

        Ok(())
    }
}
