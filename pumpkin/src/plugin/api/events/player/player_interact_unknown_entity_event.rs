use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::java::server::play::ActionType;

use super::PlayerEvent;

/// Event that is triggered when a player interacts with an entity that was not found in the world.
///
/// This commonly happens when the target entity died, was unloaded, or was otherwise removed
/// between the client sending the interaction packet and the server processing it (e.g. when
/// quickly attacking multiple mobs), and does not by itself indicate a malicious client.
/// The server takes no punitive action (such as kicking the player) for this by default; the
/// event exists so plugins can observe or react to these unknown-entity interactions.
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
