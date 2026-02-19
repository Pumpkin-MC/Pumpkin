use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's item takes durability damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemDamageEvent {
    /// The player whose item is being damaged.
    pub player: Arc<Player>,

    /// The item stack being damaged.
    pub item_stack: ItemStack,

    /// The durability damage to apply.
    pub damage: i32,
}

impl PlayerItemDamageEvent {
    /// Creates a new instance of `PlayerItemDamageEvent`.
    pub const fn new(player: Arc<Player>, item_stack: ItemStack, damage: i32) -> Self {
        Self {
            player,
            item_stack,
            damage,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerItemDamageEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
