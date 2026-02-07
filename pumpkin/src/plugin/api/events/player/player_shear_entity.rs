use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player shears an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerShearEntityEvent {
    /// The player shearing the entity.
    pub player: Arc<Player>,

    /// The entity UUID.
    pub entity_uuid: uuid::Uuid,

    /// The entity type.
    pub entity_type: String,

    /// The item used.
    pub item_stack: ItemStack,

    /// The hand used.
    pub hand: String,
}

impl PlayerShearEntityEvent {
    /// Creates a new instance of `PlayerShearEntityEvent`.
    pub const fn new(
        player: Arc<Player>,
        entity_uuid: uuid::Uuid,
        entity_type: String,
        item_stack: ItemStack,
        hand: String,
    ) -> Self {
        Self {
            player,
            entity_uuid,
            entity_type,
            item_stack,
            hand,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerShearEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
