use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player attempts to enter a bed.
///
/// If the event is cancelled, the player will not enter the bed.
///
/// Matches Bukkit's `PlayerBedEnterEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedEnterEvent {
    /// The player attempting to enter the bed.
    pub player: Arc<Player>,

    /// The position of the bed block.
    pub bed_position: BlockPos,
}

impl PlayerBedEnterEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_position: BlockPos) -> Self {
        Self {
            player,
            bed_position,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerBedEnterEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
