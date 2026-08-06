use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when an item held or worn by a player takes durability damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemDamageEvent {
    /// The player whose item is taking damage.
    pub player: Arc<Player>,

    /// The registry name of the item taking damage.
    pub item_name: String,

    /// The amount of durability damage.
    pub damage: i32,
}

impl PlayerItemDamageEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, item_name: String, damage: i32) -> Self {
        Self {
            player,
            item_name,
            damage,
            cancelled: false,
        }
    }
}
