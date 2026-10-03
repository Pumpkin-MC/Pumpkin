use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player changes their held hotbar slot.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemHeldEvent {
    /// The player who changed the held slot.
    pub player: Arc<Player>,

    /// The previous hotbar slot.
    pub previous_slot: u8,

    /// The new hotbar slot.
    pub new_slot: u8,
}

impl PlayerItemHeldEvent {
    /// Creates a new instance of `PlayerItemHeldEvent`.
    pub const fn new(player: Arc<Player>, previous_slot: u8, new_slot: u8) -> Self {
        Self {
            player,
            previous_slot,
            new_slot,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerItemHeldEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
