use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use pumpkin_util::Hand;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player consumes an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemConsumeEvent {
    /// The player consuming the item.
    pub player: Arc<Player>,

    /// The item stack being consumed.
    pub item_stack: ItemStack,

    /// The hand used to consume the item.
    pub hand: Hand,
}

impl PlayerItemConsumeEvent {
    /// Creates a new instance of `PlayerItemConsumeEvent`.
    pub const fn new(player: Arc<Player>, item_stack: ItemStack, hand: Hand) -> Self {
        Self {
            player,
            item_stack,
            hand,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerItemConsumeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
