use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player's velocity changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerVelocityEvent {
    /// The player involved.
    pub player: Arc<Player>,

    /// The new velocity vector.
    pub velocity: Vector3<f64>,
}

impl PlayerEvent for PlayerVelocityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
