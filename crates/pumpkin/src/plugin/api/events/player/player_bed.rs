use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a player enters a bed.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedEnterEvent {
    /// The player entering the bed.
    pub player: Arc<Player>,

    /// The position of the bed.
    pub bed_pos: BlockPos,
}

impl PlayerBedEnterEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_pos: BlockPos) -> Self {
        Self {
            player,
            bed_pos,
            cancelled: false,
        }
    }
}

/// An event that occurs when a player leaves a bed.
#[derive(Event, Clone)]
pub struct PlayerBedLeaveEvent {
    /// The player leaving the bed.
    pub player: Arc<Player>,

    /// The position of the bed.
    pub bed_pos: BlockPos,
}

impl PlayerBedLeaveEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_pos: BlockPos) -> Self {
        Self { player, bed_pos }
    }
}
