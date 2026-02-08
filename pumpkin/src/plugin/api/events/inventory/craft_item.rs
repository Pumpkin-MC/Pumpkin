use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

/// An event that occurs when a player crafts an item.
///
/// If the event is cancelled, the crafting will not occur.
///
/// Matches Bukkit's `CraftItemEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct CraftItemEvent {
    /// The player crafting the item.
    pub player: Arc<Player>,

    /// The resulting item being crafted.
    pub result: ItemStack,
}

impl CraftItemEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, result: ItemStack) -> Self {
        Self {
            player,
            result,
            cancelled: false,
        }
    }
}
