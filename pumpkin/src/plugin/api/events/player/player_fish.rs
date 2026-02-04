use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player fishes.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerFishEvent {
    /// The player who is fishing.
    pub player: Arc<Player>,

    /// The UUID of the caught entity, if any.
    pub caught_uuid: Option<uuid::Uuid>,

    /// The caught entity type (registry key).
    pub caught_type: String,

    /// The UUID of the fishing hook.
    pub hook_uuid: uuid::Uuid,

    /// The fish event state.
    pub state: String,

    /// The hand used for fishing.
    pub hand: String,

    /// Experience to drop.
    pub exp_to_drop: i32,
}

impl PlayerFishEvent {
    /// Creates a new instance of `PlayerFishEvent`.
    pub fn new(
        player: Arc<Player>,
        caught_uuid: Option<uuid::Uuid>,
        hook_uuid: uuid::Uuid,
        caught_type: String,
        state: String,
        hand: String,
        exp_to_drop: i32,
    ) -> Self {
        Self {
            player,
            caught_uuid,
            hook_uuid,
            caught_type,
            state,
            hand,
            exp_to_drop,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerFishEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
