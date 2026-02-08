use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player leaves a bed.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `PlayerBedLeaveEvent`.
#[derive(Event, Clone)]
pub struct PlayerBedLeaveEvent {
    /// The player leaving the bed.
    pub player: Arc<Player>,

    /// The position of the bed block.
    pub bed_position: BlockPos,
}

impl PlayerBedLeaveEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_position: BlockPos) -> Self {
        Self {
            player,
            bed_position,
        }
    }
}

impl PlayerEvent for PlayerBedLeaveEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
