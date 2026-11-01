use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// The type of animation performed by a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAnimationType {
    ArmSwingMain,
    ArmSwingOff,
    LeaveBed,
}

/// An event that occurs when a player performs an animation.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerAnimationEvent {
    /// The player performing the animation.
    pub player: Arc<Player>,

    /// The type of animation.
    pub animation_type: PlayerAnimationType,
}

impl PlayerAnimationEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, animation_type: PlayerAnimationType) -> Self {
        Self {
            player,
            animation_type,
            cancelled: false,
        }
    }
}
