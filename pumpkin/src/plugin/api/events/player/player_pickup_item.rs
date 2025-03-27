use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_data::item::Item;

use super::PlayerEvent;

/// An event that occurs when a player picks up an item.
///
/// If the event is cancelled, the item will not be picked up.
///
/// This event contains information about the player, the item being picked up, and the amount of the item that was picked up.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPickupItemEvent {
    /// The player who is picking up the item.
    pub player: Arc<Player>,

    /// The item being picked up.
    pub item: Item,

    /// The amount of the item that was picked up.
    pub amount: u32,
}

impl PlayerPickupItemEvent {
    /// Creates a new instance of `PlayerPickupItemEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is picking up the item.
    /// - `item`: The item being picked up.
    /// - `amount`: The amount of the item that was picked up.
    ///
    /// # Returns
    /// A new instance of `PlayerPickupItemEvent`.
    pub fn new(player: Arc<Player>, item: Item, amount: u32) -> Self {
        Self {
            player,
            item,
            amount,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerPickupItemEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
