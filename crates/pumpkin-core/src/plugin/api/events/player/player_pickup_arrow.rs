use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player picks up an arrow.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPickupArrowEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Arrow entity ID.
    pub arrow_id: i32,
}

impl PlayerPickupArrowEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, arrow_id: i32) -> Self {
        Self {
            player,
            arrow_id,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerPickupArrowEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
