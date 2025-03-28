use super::PlayerEvent;
use crate::entity::player::Player;
use crate::world::GetBlockError;
use pumpkin_data::block::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::block::BlockDirection;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

/// An event that occurs when a `Player` interacts with a `Block` using their hand.
/// This event does not consider interactions through block movement, eg pressure plates, tripwire hooks, sculk sensors etc.
///
/// If the event is cancelled, the interaction  will not happen.
///
/// This event contains information about the player, whether the player is sneaking or not, the `Block` they are interacting with,
/// the `ItemStack` they are interacting using, the block face (`BlockDirection`) they are interacting with.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractEvent {
    /// The player who attempted to interact.
    pub player: Arc<Player>,

    /// Is the player sneaking?
    pub sneaking: bool,

    /// The block the player is interacting with.
    pub block: Result<Block, GetBlockError>,

    /// The face of the block the player is interacting with
    pub block_direction: BlockDirection,

    /// The ItemStack the player is interacting using
    pub item_stack: Arc<Option<ItemStack>>,
}

impl PlayerInteractEvent {
    /// Creates a new instance of `PlayerInteractEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who interacted.
    /// - `sneaking`: Is the player sneaking?
    /// - `block`: The block the player is interacting with.
    /// - `block_face`: The face of the block the player is interacting with.
    /// - `item`: The `ItemStack` the player is interacting using.
    /// - `cancelled`: A boolean indicating whether the interaction should be cancelled.
    ///
    /// # Returns
    /// A new instance of `PlayerInteractEvent`.
    pub fn new(
        player: Arc<Player>,
        sneaking: bool,
        block: Result<Block, GetBlockError>,
        block_direction: BlockDirection,
        item_stack: Arc<Option<ItemStack>>,
        cancelled: bool,
    ) -> Self {
        Self {
            player,
            sneaking,
            block,
            block_direction,
            item_stack,
            cancelled,
        }
    }
}

impl PlayerEvent for PlayerInteractEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
