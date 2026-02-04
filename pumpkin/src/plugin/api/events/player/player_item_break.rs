use pumpkin_macros::Event;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's item breaks.
#[derive(Event, Clone)]
pub struct PlayerItemBreakEvent {
    /// The player whose item broke.
    pub player: Arc<Player>,

    /// The item stack that broke.
    pub item_stack: ItemStack,
}

impl PlayerItemBreakEvent {
    /// Creates a new instance of `PlayerItemBreakEvent`.
    pub const fn new(player: Arc<Player>, item_stack: ItemStack) -> Self {
        Self { player, item_stack }
    }
}

impl PlayerEvent for PlayerItemBreakEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
