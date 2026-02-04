pub mod player_change_world;
pub mod player_chat;
pub mod player_command_preprocess;
pub mod player_command_send;
pub mod player_custom_payload;
pub mod player_drop_item;
pub mod player_edit_book;
pub mod player_egg_throw;
pub mod player_exp_change;
pub mod player_fish;
pub mod player_interact_at_entity;
pub mod player_interact_entity;
pub mod player_gamemode_change;
pub mod player_interact_entity_event;
pub mod player_interact_event;
pub mod player_interact_unknown_entity_event;
pub mod player_join;
pub mod player_leave;
pub mod player_login;
pub mod player_pre_login;
pub mod player_advancement_done;
pub mod player_animation;
pub mod player_armor_stand_manipulate;
pub mod player_bed_enter;
pub mod player_bed_leave;
pub mod player_bucket_empty;
pub mod player_bucket_fill;
pub mod player_bucket_entity;
pub mod player_changed_main_hand;
pub mod player_register_channel;
pub mod player_unregister_channel;
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
