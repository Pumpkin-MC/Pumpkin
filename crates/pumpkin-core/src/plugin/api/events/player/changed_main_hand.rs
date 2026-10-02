use pumpkin_macros::Event;
use pumpkin_util::Hand;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player changes their main hand.
#[derive(Event, Clone)]
pub struct PlayerChangedMainHandEvent {
    /// The player whose main hand changed.
    pub player: Arc<Player>,

    /// The player's new main hand.
    pub main_hand: Hand,
}

impl PlayerChangedMainHandEvent {
    /// Creates a new instance of `PlayerChangedMainHandEvent`.
    pub const fn new(player: Arc<Player>, main_hand: Hand) -> Self {
        Self { player, main_hand }
    }
}

impl PlayerEvent for PlayerChangedMainHandEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
