use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::java::server::play::ActionType;

use super::PlayerEvent;

/// Event that is triggered when a player interacts with an entity that was not found in the world.
///
/// This can occur when the target entity has been removed or is otherwise unknown to the server.
/// Plugins can cancel it if they want to override the default handling.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractUnknownEntityEvent {
    /// The player who performed the interaction.
    pub player: Arc<Player>,

    /// The entity ID that was targeted.
    pub entity_id: i32,

    /// The type of interaction (Interact, Attack, or `InteractAt`).
    pub action: ActionType,
}

impl PlayerInteractUnknownEntityEvent {
    pub fn new(player: &Arc<Player>, entity_id: i32, action: ActionType) -> Self {
        Self {
            player: Arc::clone(player),
            entity_id,
            action,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerInteractUnknownEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
