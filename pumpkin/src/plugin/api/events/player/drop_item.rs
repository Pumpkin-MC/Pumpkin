use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player drops an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerDropItemEvent {
    /// The player who dropped the item.
    pub player: Arc<Player>,

    /// The UUID of the dropped item entity.
    pub item_uuid: uuid::Uuid,

    /// The item stack being dropped.
    pub item_stack: ItemStack,
}

impl PlayerDropItemEvent {
    /// Creates a new instance of `PlayerDropItemEvent`.
    pub const fn new(player: Arc<Player>, item_uuid: uuid::Uuid, item_stack: ItemStack) -> Self {
        Self {
            player,
            item_uuid,
            item_stack,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerDropItemEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
