use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_util::math::vector3::Vector3;

use super::PlayerEvent;

/// An event that occurs when a player enters a bed.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedEnterEvent {
    /// The player who is entering the bed.
    pub player: Arc<Player>,

    /// The bed position.
    pub bed_position: Vector3<f64>,
}

impl PlayerBedEnterEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_position: Vector3<f64>) -> Self {
        Self {
            player,
            bed_position,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerBedEnterEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
