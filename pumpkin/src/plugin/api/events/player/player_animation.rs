use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player performs an animation (e.g., arm swing).
#[derive(Event, Clone)]
pub struct PlayerAnimationEvent {
    /// The player who performed the animation.
    pub player: Arc<Player>,

    /// Animation type, e.g. "ARM_SWING".
    pub animation_type: String,
}

impl PlayerAnimationEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, animation_type: String) -> Self {
        Self {
            player,
            animation_type,
        }
    }
}

impl PlayerEvent for PlayerAnimationEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
