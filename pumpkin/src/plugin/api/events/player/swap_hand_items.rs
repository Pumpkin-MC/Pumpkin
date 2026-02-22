use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player swaps main hand and off hand items.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerSwapHandItemsEvent {
    /// The player swapping items.
    pub player: Arc<Player>,

    /// The item in the main hand after swap.
    pub main_hand_item: ItemStack,

    /// The item in the off hand after swap.
    pub off_hand_item: ItemStack,
}

impl PlayerSwapHandItemsEvent {
    /// Creates a new instance of `PlayerSwapHandItemsEvent`.
    pub const fn new(
        player: Arc<Player>,
        main_hand_item: ItemStack,
        off_hand_item: ItemStack,
    ) -> Self {
        Self {
            player,
            main_hand_item,
            off_hand_item,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerSwapHandItemsEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
