use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_MOTION;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{VarInt, codec::velocity::Velocity};

/// Updates the velocity of an entity.
///
/// This packet informs the client of a sudden change in an entity's movement,
/// such as knockback from an attack, explosions, or being launched by a piston.
#[derive(Serialize)]
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
