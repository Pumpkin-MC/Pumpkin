use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

/// An event that occurs when a player clicks a slot in an inventory.
///
/// If the event is cancelled, the click action will not occur.
///
/// Matches Bukkit's `InventoryClickEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryClickEvent {
    /// The player who clicked.
    pub player: Arc<Player>,

    /// The slot number that was clicked (-999 for outside click).
    pub slot: i32,

    /// The click type (e.g. "LEFT", "RIGHT", "SHIFT_LEFT", "SHIFT_RIGHT", "MIDDLE", "DROP").
    pub click_type: String,
}

impl InventoryClickEvent {
    #[must_use]
    pub fn new(player: Arc<Player>, slot: i32, click_type: String) -> Self {
        Self {
            player,
            slot,
            click_type,
            cancelled: false,
        }
    }
}
