use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;

use super::PlayerEvent;

/// An event that occurs when a player consumes an item (food, potion, milk bucket, etc.).
///
/// If the event is cancelled, the item will not be consumed.
///
/// Matches Bukkit's `PlayerItemConsumeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemConsumeEvent {
    /// The player who is consuming the item.
    pub player: Arc<Player>,

    /// The item being consumed.
    pub item: ItemStack,
}

impl PlayerItemConsumeEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, item: ItemStack) -> Self {
        Self {
            player,
            item,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerItemConsumeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
