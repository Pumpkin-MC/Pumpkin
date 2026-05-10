use std::pin::Pin;

use crate::{BlockStateId, generation::proto_chunk::GenerationCache};
use bitflags::bitflags;
use pumpkin_data::{Block, BlockState, chunk::Biome};
use pumpkin_util::math::position::BlockPos;
use thiserror::Error;

bitflags! {
    /// Flags used to control the side effects of a block state change.
    /// These match the internal bitmask used by Minecraft's `setBlockState` method
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BlockFlags: u32 {
        /// Causes a neighbor update to be sent to surrounding blocks.
        /// This is what makes observers detect changes or redstone components react
        const NOTIFY_NEIGHBORS                      = 0b000_0000_0001;
        /// Notifies listeners (like clients) that the block has changed.
        /// On the server, this triggers a packet to be sent to players in range
        const NOTIFY_LISTENERS                      = 0b000_0000_0010;
        /// Combines neighbor notification and listener notification.
        /// This is the "standard" update used for most block placements
        const NOTIFY_ALL                            = 0b000_0000_0011;
        /// Forces the block state to be set even if it matches the current state
        /// Used by items like the Debug Stick to bypass "virtual" change checks
        const FORCE_STATE                           = 0b000_0000_0100;
        /// Prevents the previous block from dropping items when it is replaced
        /// Commonly used when a block is "transformed" rather than destroyed
        const SKIP_DROPS                            = 0b000_0000_1000;
        /// Signals that the block is being moved (usually by a piston)
        /// This prevents certain "on-break" logic from firing until the move is complete
        const MOVED                                 = 0b000_0001_0000;
        /// Prevents redstone wire from re-calculating its shape/power immediately
        /// Used during massive redstone updates to reduce calculation lag
        const SKIP_REDSTONE_WIRE_STATE_REPLACEMENT  = 0b000_0010_0000;
        /// If set, the `on_replaced` callback for block entities (containers) is skipped
        /// Useful if you are moving a Block Entity and don't want it to drop its contents yet
        const SKIP_BLOCK_ENTITY_REPLACED_CALLBACK   = 0b000_0100_0000;
        /// Prevents the `on_added` logic from firing for the new block state
        /// Use this to avoid recursive placement loops or unnecessary initialization
        const SKIP_BLOCK_ADDED_CALLBACK             = 0b000_1000_0000;
    }
}

#[derive(Debug, Error)]
pub enum GetBlockError {
    InvalidBlockId,
    BlockOutOfWorldBounds,
}

impl std::fmt::Display for GetBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type WorldFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait WorldPortalExt: Send + Sync {
    fn can_place_at(
        &self,
        block: &Block,
        state: &BlockState,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> bool;

    fn spawn_mobs_for_chunk_generation(
        &self,
        cache: &mut dyn GenerationCache,
        biome: &'static Biome,
        chunk_x: i32,
        chunk_z: i32,
    );
}

pub trait BlockAccessor: Send + Sync {
    fn get_block<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static Block> + Send + 'a>>;

    fn get_block_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static BlockState> + Send + 'a>>;

    fn get_block_state_id<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = BlockStateId> + Send + 'a>>;

    fn get_block_and_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = (&'static Block, &'static BlockState)> + Send + 'a>>;
}
