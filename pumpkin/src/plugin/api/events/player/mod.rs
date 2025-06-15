pub mod player_change_world;
pub mod player_chat;
pub mod player_command_send;
pub mod player_gamemode_change;
pub mod player_interact_event;
pub mod player_join;
pub mod player_leave;
pub mod player_login;
pub mod player_move;
pub mod player_teleport;

use std::sync::Arc;

use crate::entity::player::Player;

/// A trait representing events related to players.
///
/// This trait provides a method to retrieve the player associated with the event.
pub trait PlayerEvent: Send + Sync {
    /// Retrieves a reference to the player associated with the event.
    ///
    /// # Returns
    /// A reference to the `Arc<Player>` involved in the event.
    fn get_player(&self) -> &Arc<Player>;
}
