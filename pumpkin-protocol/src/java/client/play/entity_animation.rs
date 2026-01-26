use pumpkin_data::packet::clientbound::PLAY_ANIMATE;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Triggers a specific animation for an entity that is visible to the client.
///
/// This is primarily used for player-driven animations like swinging an arm
/// or showing damage, but it can apply to other entities as well.
#[derive(Serialize)]
#[java_packet(PLAY_ANIMATE)]
pub struct CEntityAnimation {
    /// The Entity ID of the entity performing the animation.
    pub entity_id: VarInt,
    /// The ID of the animation to play.
    /// See the table below for standard values.
    pub animation: u8,
}

impl CEntityAnimation {
    pub fn new(entity_id: VarInt, animation: Animation) -> Self {
        Self {
            entity_id,
            animation: animation as u8,
        }
    }
}

#[derive(Debug)]
pub enum Animation {
    SwingMainArm,
    LeaveBed = 2,
    SwingOffhand,
    CriticalEffect,
    MagicCriticaleffect,
}
