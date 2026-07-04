use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player drops an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerDropItemEvent {
    /// The player who dropped the item.
    pub player: Arc<Player>,

    /// The item stack being dropped.
    pub item_stack: ItemStack,

    /// Whether the original drop action targeted the full held stack.
    pub drop_full_stack: bool,
}

impl PlayerDropItemEvent {
    /// Creates a new instance of `PlayerDropItemEvent`.
    pub const fn new(player: Arc<Player>, item_stack: ItemStack, drop_full_stack: bool) -> Self {
        Self {
            player,
            item_stack,
            drop_full_stack,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerDropItemEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
