pub mod bedrock_form_response;
pub mod changed_main_hand;
pub mod custom_click_action;
pub mod egg_throw;
pub mod exp_change;
pub mod fish;
pub mod inventory_close;
pub mod inventory_interact;
pub mod item_held;
pub mod player_bed;
pub mod player_bucket;
pub mod player_change_world;
pub mod player_chat;
pub mod player_command_send;
pub mod player_custom_payload;
pub mod player_drop_item;
pub mod player_gamemode_change;
pub mod player_interact_entity_event;
pub mod player_interact_event;
pub mod player_interact_unknown_entity_event;
pub mod player_item_consume;
pub mod player_item_damage;
pub mod player_join;
pub mod player_leave;
pub mod player_login;
pub mod player_move;
pub mod player_permission_check;
pub mod player_respawn;
pub mod player_teleport;
pub mod player_toggle_flight_event;
pub mod player_toggle_sneak_event;
pub mod player_toggle_sprint_event;

pub use bedrock_form_response::*;
pub use changed_main_hand::*;
pub use custom_click_action::*;
pub use egg_throw::*;
pub use exp_change::*;
pub use fish::*;
pub use inventory_close::*;
pub use inventory_interact::*;
pub use item_held::*;
pub use player_bed::*;
pub use player_bucket::*;
pub use player_change_world::*;
pub use player_chat::*;
pub use player_command_send::*;
pub use player_custom_payload::*;
pub use player_drop_item::*;
pub use player_gamemode_change::*;
pub use player_interact_entity_event::*;
pub use player_interact_event::*;
pub use player_interact_unknown_entity_event::*;
pub use player_item_consume::*;
pub use player_item_damage::*;
pub use player_join::*;
pub use player_leave::*;
pub use player_login::*;
pub use player_move::*;
pub use player_permission_check::*;
pub use player_respawn::*;
pub use player_teleport::*;
pub use player_toggle_flight_event::*;
pub use player_toggle_sneak_event::*;
pub use player_toggle_sprint_event::*;

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
