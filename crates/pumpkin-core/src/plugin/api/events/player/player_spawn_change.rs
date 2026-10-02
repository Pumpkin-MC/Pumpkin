use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player's spawn location changes (e.g. bed/respawn anchor).
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerSpawnChangeEvent {
    /// The player whose spawn changed.
    pub player: Arc<Player>,

    /// The new spawn position, if set.
    pub new_spawn: Option<BlockPos>,

    /// Whether this spawn point is forced.
    pub forced: bool,
}

impl PlayerEvent for PlayerSpawnChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
