pub mod block_break;
pub mod block_burn;
pub mod block_can_build;
pub mod block_damage;
pub mod block_dispense;
pub mod block_explode;
pub mod block_fade;
pub mod block_form;
pub mod block_from_to;
pub mod block_grow;
pub mod block_ignite;
pub mod block_physics;
pub mod block_piston;
pub mod block_place;
pub mod block_redstone;
pub mod note_play;
pub mod sign_change;
pub mod sponge_absorb;
pub mod tnt_prime;

pub use block_break::*;
pub use block_burn::*;
pub use block_can_build::*;
pub use block_damage::*;
pub use block_dispense::*;
pub use block_explode::*;
pub use block_fade::*;
pub use block_form::*;
pub use block_from_to::*;
pub use block_grow::*;
pub use block_ignite::*;
pub use block_physics::*;
pub use block_piston::*;
pub use block_place::*;
pub use block_redstone::*;
pub use note_play::*;
pub use sign_change::*;
pub use sponge_absorb::*;
pub use tnt_prime::*;

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
