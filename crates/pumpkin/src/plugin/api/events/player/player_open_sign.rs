use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a sign editor is opened for a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerOpenSignEvent {
    /// The player opening the sign.
    pub player: Arc<Player>,

    /// The sign block position.
    pub block_pos: BlockPos,

    /// Whether the front side is being edited.
    pub is_front: bool,
}

impl PlayerEvent for PlayerOpenSignEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
