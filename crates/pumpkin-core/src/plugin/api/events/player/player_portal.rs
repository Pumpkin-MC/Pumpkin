use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player uses a portal.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPortalEvent {
    /// The player using the portal.
    pub player: Arc<Player>,

    /// The portal position where the player entered.
    pub from_pos: BlockPos,

    /// The destination position, if known.
    pub to_pos: Option<BlockPos>,
}

impl PlayerEvent for PlayerPortalEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
