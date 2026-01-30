use pumpkin_data::packet::clientbound::PLAY_HURT_ANIMATION;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

/// Triggers the "hurt" visual effect on an entity.
///
/// This packet causes the entity to turn red and perform a directional
/// camera shake or model tilt. It is typically sent immediately after
/// an entity's health is reduced.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_HURT_ANIMATION)]
pub struct CHurtAnimation {
    /// The Entity ID of the entity that was hurt.
    pub entity_id: VarInt,
    /// The yaw (direction) from which the damage originated.
    /// This determines the direction the entity's model tilts.
    pub yaw: f32,
}

impl CHurtAnimation {
    #[must_use]
    pub const fn new(entity_id: VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}
