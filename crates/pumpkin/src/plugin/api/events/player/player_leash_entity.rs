use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player leashes an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLeashEntityEvent {
    /// The player leashing the entity.
    pub player: Arc<Player>,

    /// The ID of the leashed entity.
    pub entity_id: i32,

    /// The ID of the holder entity or fence.
    pub holder_id: i32,
}

impl PlayerEvent for PlayerLeashEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
