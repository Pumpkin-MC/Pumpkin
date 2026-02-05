pub mod block_break;
pub mod block_burn;
pub mod block_can_build;
pub mod block_damage;
pub mod block_damage_abort;
pub mod block_dispense;
pub mod block_drop_item;
pub mod block_explode;
pub mod block_fade;
pub mod block_fertilize;
pub mod block_form;
pub mod block_from_to;
pub mod block_grow;
pub mod block_ignite;
pub mod block_spread;
pub mod block_place;

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
