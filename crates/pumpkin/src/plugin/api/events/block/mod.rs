pub mod bell_resonate;
pub mod bell_ring;
pub mod block_break;
pub mod block_brush;
pub mod block_burn;
pub mod block_can_build;
pub mod block_cook;
pub mod block_damage;
pub mod block_damage_abort;
pub mod block_dispense;
pub mod block_dispense_armor;
pub mod block_dispense_loot;
pub mod block_drop_item;
pub mod block_exp;
pub mod block_explode;
pub mod block_fade;
pub mod block_fertilize;
pub mod block_form;
pub mod block_from_to;
pub mod block_grow;
pub mod block_ignite;
pub mod block_multi_place;
pub mod block_physics;
pub mod block_piston;
pub mod block_place;
pub mod block_receive_game;
pub mod block_redstone;
pub mod block_shear_entity;
pub mod block_spread;
pub mod brewing_start;
pub mod campfire_start;
pub mod cauldron_level_change;
pub mod crafter_craft;
pub mod entity_block_form;
pub mod fluid_level_change;
pub mod inventory_block_start;
pub mod leaves_decay;
pub mod moisture_change;
pub mod note_play;
pub mod sculk_bloom;
pub mod sign_change;
pub mod sponge_absorb;
pub mod tnt_prime;
pub mod vault_display_item;

pub use bell_resonate::*;
pub use bell_ring::*;
pub use block_break::*;
pub use block_brush::*;
pub use block_burn::*;
pub use block_can_build::*;
pub use block_cook::*;
pub use block_damage::*;
pub use block_damage_abort::*;
pub use block_dispense::*;
pub use block_dispense_armor::*;
pub use block_dispense_loot::*;
pub use block_drop_item::*;
pub use block_exp::*;
pub use block_explode::*;
pub use block_fade::*;
pub use block_fertilize::*;
pub use block_form::*;
pub use block_from_to::*;
pub use block_grow::*;
pub use block_ignite::*;
pub use block_multi_place::*;
pub use block_physics::*;
pub use block_piston::*;
pub use block_place::*;
pub use block_receive_game::*;
pub use block_redstone::*;
pub use block_shear_entity::*;
pub use block_spread::*;
pub use brewing_start::*;
pub use campfire_start::*;
pub use cauldron_level_change::*;
pub use crafter_craft::*;
pub use entity_block_form::*;
pub use fluid_level_change::*;
pub use inventory_block_start::*;
pub use leaves_decay::*;
pub use moisture_change::*;
pub use note_play::*;
pub use sculk_bloom::*;
pub use sign_change::*;
pub use sponge_absorb::*;
pub use tnt_prime::*;
pub use vault_display_item::*;

use pumpkin_data::Block;

/// A trait representing events related to blocks.
///
/// This trait provides a method to retrieve the block associated with the event.
pub trait BlockEvent: Send + Sync {
    /// Retrieves a reference to the block associated with the event.
    ///
    /// # Returns
    /// A reference to the `Block` involved in the event.
    fn get_block(&self) -> &Block;
}
