use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;

use super::PlayerEvent;

/// An event that occurs when a player drops an item.
///
/// If the event is cancelled, the item will not be dropped.
///
/// This event contains information about the player, and the item being dropped.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerDropItemEvent {
    /// The player who is dropping the item.
    pub player: Arc<Player>,

    /// The item being dropped.
    pub item_stack: ItemStack,
}

impl PlayerDropItemEvent {
    /// Creates a new instance of `PlayerDropItemEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is dropping the item.
    /// - `item_stack`: The `ItemStack` being dropped.
    ///
    /// # Returns
    /// A new instance of `PlayerDropItemEvent`.
    pub fn new(player: Arc<Player>, item_stack: ItemStack) -> Self {
        Self {
            player,
            item_stack,
            cancelled: false,
        }
    }

    /// Gets the item being dropped.
    ///
    /// # Returns
    /// A reference to the `Item` being dropped.
    #[must_use]
    pub fn get_item(&self) -> &Item {
        &self.item_stack.item
    }

    /// Gets the count of items being dropped.
    ///
    /// # Returns
    /// The number of items being dropped.
    #[must_use]
    pub fn get_count(&self) -> u8 {
        self.item_stack.item_count
    }
}

impl PlayerEvent for PlayerDropItemEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
