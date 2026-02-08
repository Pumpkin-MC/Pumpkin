use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player changes the item held in their hand.
///
/// If the event is cancelled, the held item slot change will not occur.
///
/// Matches Bukkit's `PlayerItemHeldEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemHeldEvent {
    /// The player changing their held item.
    pub player: Arc<Player>,

    /// The previous slot index (0-8).
    pub previous_slot: u8,

    /// The new slot index (0-8).
    pub new_slot: u8,
}

impl PlayerItemHeldEvent {
    #[must_use]
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
