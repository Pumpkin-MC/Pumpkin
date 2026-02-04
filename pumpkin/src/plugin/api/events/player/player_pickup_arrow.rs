use pumpkin_macros::{cancellable, Event};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player picks up an arrow.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPickupArrowEvent {
    /// The player picking up the arrow.
    pub player: Arc<Player>,

    /// The arrow entity UUID.
    pub arrow_uuid: uuid::Uuid,

    /// The item entity UUID.
    pub item_uuid: uuid::Uuid,

    /// The item stack being picked up.
    pub item_stack: ItemStack,

    /// Remaining items in the pickup.
    pub remaining: i32,
}

impl PlayerPickupArrowEvent {
    /// Creates a new instance of `PlayerPickupArrowEvent`.
    pub fn new(
        player: Arc<Player>,
        arrow_uuid: uuid::Uuid,
        item_uuid: uuid::Uuid,
        item_stack: ItemStack,
        remaining: i32,
    ) -> Self {
        Self {
            player,
            arrow_uuid,
            item_uuid,
            item_stack,
            remaining,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerPickupArrowEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
