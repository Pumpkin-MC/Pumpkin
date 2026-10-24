use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player's item is mended with experience.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemMendEvent {
    /// The player owning the item.
    pub player: Arc<Player>,

    /// The name of the item repaired.
    pub item_name: String,

    /// The durability points repaired.
    pub repair_amount: i32,

    /// The experience points consumed.
    pub exp_consumed: i32,
}

impl PlayerEvent for PlayerItemMendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
