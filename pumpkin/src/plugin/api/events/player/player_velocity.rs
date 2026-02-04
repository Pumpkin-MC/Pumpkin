use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's velocity changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerVelocityEvent {
    /// The player whose velocity changed.
    pub player: Arc<Player>,

    /// The new velocity.
    pub velocity: Vector3<f64>,
}

impl PlayerVelocityEvent {
    /// Creates a new instance of `PlayerVelocityEvent`.
    pub const fn new(player: Arc<Player>, velocity: Vector3<f64>) -> Self {
        Self {
            player,
            velocity,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerVelocityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
