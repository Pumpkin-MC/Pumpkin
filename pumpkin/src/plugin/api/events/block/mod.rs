pub mod block_break;
pub mod block_burn;
pub mod block_can_build;
pub mod block_place;

use pumpkin_world::block::registry::Block;

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
