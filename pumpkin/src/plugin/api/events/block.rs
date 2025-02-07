use pumpkin_macros::{cancellable, Event};
use pumpkin_world::block::registry::Block;
use std::sync::Arc;

use crate::entity::player::Player;

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

/// An event that occurs when a block is broken.
///
/// This event contains information about the player breaking the block, the block itself,
/// the experience gained, and whether the block should drop items.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockBreakEvent {
    /// The player who is breaking the block, if applicable.
    pub player: Option<Arc<Player>>,

    /// The block that is being broken.
    pub block: Block,

    /// The amount of experience gained from breaking the block.
    pub exp: u32,

    /// A boolean indicating whether the block should drop items.
    pub drop: bool,
}

impl BlockBreakEvent {
    /// Creates a new instance of `BlockBreakEvent`.
    ///
    /// # Arguments
    /// - `player`: An optional reference to the player breaking the block.
    /// - `block`: The block that is being broken.
    /// - `exp`: The amount of experience gained from breaking the block.
    /// - `drop`: A boolean indicating whether the block should drop items.
    ///
    /// # Returns
    /// A new instance of `BlockBreakEvent`.
    #[must_use]
    pub fn new(player: Option<Arc<Player>>, block: Block, exp: u32, drop: bool) -> Self {
        Self {
            player,
            block,
            exp,
            drop,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockBreakEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

/// An event that occurs when a block is burned.
///
/// This event contains information about the block that ignited the fire and the block that is burning.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockBurnEvent {
    /// The block that is igniting the fire.
    pub igniting_block: Block,

    /// The block that is burning.
    pub block: Block,
}

impl BlockEvent for BlockBurnEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

/// An event that occurs when a player attempts to build on a block.
///
/// This event contains information about the block to build, whether building is allowed,
/// the player attempting to build, and the block being built upon.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockCanBuildEvent {
    /// The block that the player is attempting to build.
    pub block_to_build: Block,

    /// A boolean indicating whether building is allowed.
    pub buildable: bool,

    /// The player attempting to build.
    pub player: Arc<Player>,

    /// The block being built upon.
    pub block: Block,
}

impl BlockEvent for BlockCanBuildEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

/// An event that occurs when a block is placed.
///
/// This event contains information about the player placing the block, the block being placed,
/// the block being placed against, and whether the player can build.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPlaceEvent {
    /// The player placing the block.
    pub player: Arc<Player>,

    /// The block that is being placed.
    pub block_placed: Block,

    /// The block that the new block is being placed against.
    pub block_placed_against: Block,

    /// A boolean indicating whether the player can build.
    pub can_build: bool,
}

impl BlockEvent for BlockPlaceEvent {
    fn get_block(&self) -> &Block {
        &self.block_placed
    }
}
