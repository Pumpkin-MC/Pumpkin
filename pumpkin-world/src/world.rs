use std::pin::Pin;
use std::sync::Arc;

use crate::block::entities::BlockEntity;
use crate::{BlockStateId, inventory::Inventory};
use bitflags::bitflags;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
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

pub trait SimpleWorld: BlockAccessor + Send + Sync {
    fn set_block_state(
        self: Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> WorldFuture<'_, BlockStateId>;

    fn update_neighbor<'a>(
        self: Arc<Self>,
        neighbor_block_pos: &'a BlockPos,
        source_block: &'a pumpkin_data::Block,
    ) -> WorldFuture<'a, ()>;

    fn update_neighbors(
        self: Arc<Self>,
        block_pos: &BlockPos,
        except: Option<BlockDirection>,
    ) -> WorldFuture<'_, ()>;

    fn is_space_empty(&self, bounding_box: BoundingBox) -> WorldFuture<'_, bool>;

    fn spawn_from_type(
        self: Arc<Self>,
        entity_type: &'static EntityType,
        position: Vector3<f64>,
    ) -> WorldFuture<'static, ()>;

    fn add_synced_block_event(&self, pos: BlockPos, r#type: u8, data: u8) -> WorldFuture<'_, ()>;

    fn sync_world_event(
        &self,
        world_event: WorldEvent,
        position: BlockPos,
        data: i32,
    ) -> WorldFuture<'_, ()>;

    fn remove_block_entity<'a>(&'a self, block_pos: &'a BlockPos) -> WorldFuture<'a, ()>;

    fn get_block_entity<'a>(
        &'a self,
        block_pos: &'a BlockPos,
    ) -> WorldFuture<'a, Option<Arc<dyn BlockEntity>>>;

    fn get_world_age(&self) -> WorldFuture<'_, i64>;

    fn play_sound<'a>(
        &'a self,
        sound: Sound,
        category: SoundCategory,
        position: &'a Vector3<f64>,
    ) -> WorldFuture<'a, ()>;

    fn play_sound_fine<'a>(
        &'a self,
        sound: Sound,
        category: SoundCategory,
        position: &'a Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) -> WorldFuture<'a, ()>;

    /* ItemScatterer */
    fn scatter_inventory<'a>(
        self: Arc<Self>,
        position: &'a BlockPos,
        inventory: &'a Arc<dyn Inventory>,
    ) -> WorldFuture<'a, ()>;
}

pub trait BlockRegistryExt: Send + Sync {
    fn can_place_at(
        &self,
        block: &Block,
        state: &BlockState,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> bool;
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
