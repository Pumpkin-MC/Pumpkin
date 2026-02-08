use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;

use super::PlayerEvent;

/// An event that occurs when a player drops an item from their inventory.
///
/// If the event is cancelled, the item will not be dropped.
///
/// Matches Bukkit's `PlayerDropItemEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerDropItemEvent {
    /// The player who is dropping the item.
    pub player: Arc<Player>,

    /// The item being dropped.
    pub item: ItemStack,
}

impl PlayerDropItemEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, item: ItemStack) -> Self {
        Self {
            player,
            item,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerDropItemEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
