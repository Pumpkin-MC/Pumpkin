use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player gains experience points.
#[derive(Event, Clone)]
pub struct PlayerExpChangeEvent {
    /// The player gaining experience.
    pub player: Arc<Player>,

    /// The amount of experience to add.
    pub amount: i32,
}

impl PlayerExpChangeEvent {
    /// Creates a new instance of `PlayerExpChangeEvent`.
    pub const fn new(player: Arc<Player>, amount: i32) -> Self {
        Self { player, amount }
    }
}

impl PlayerEvent for PlayerExpChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
