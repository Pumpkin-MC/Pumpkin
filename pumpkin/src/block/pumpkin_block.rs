use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::BlockIsReplacing;

pub trait BlockMetadata {
    fn namespace(&self) -> &'static str;
    fn ids(&self) -> &'static [&'static str];
    fn names(&self) -> Vec<String> {
        self.ids()
            .iter()
            .map(|f| format!("{}:{}", self.namespace(), f))
            .collect()
    }
}

#[async_trait]
pub trait PumpkinBlock: Send + Sync {
    async fn normal_use(&self, _args: NormalUseArgs<'_>) {}

    async fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        BlockActionResult::Continue
    }

    async fn on_entity_collision(&self, _args: OnEntityCollisionArgs<'_>) {}

    fn should_drop_items_on_explosion(&self) -> bool {
        true
    }

    async fn explode(&self, _args: ExplodeArgs<'_>) {}

    /// Handles the block event, which is an event specific to a block with an integer ID and data.
    ///
    /// returns whether the event was handled successfully
    async fn on_synced_block_event(&self, _args: OnSyncedBlockEventArgs<'_>) -> bool {
        false
    }

    /// getPlacementState in source code
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        args.block.default_state.id
    }

    async fn random_tick(&self, _args: RandomTickArgs<'_>) {}

    async fn can_place_at(&self, _args: CanPlaceAtArgs<'_>) -> bool {
        true
    }

    async fn can_update_at(&self, _args: CanUpdateAtArgs<'_>) -> bool {
        false
    }

    /// onBlockAdded in source code
    async fn placed(&self, _args: PlacedArgs<'_>) {}

    async fn player_placed(&self, _args: PlayerPlacedArgs<'_>) {}

    async fn broken(&self, _args: BrokenArgs<'_>) {}

    async fn on_neighbor_update(&self, _args: OnNeighborUpdateArgs<'_>) {}

    /// Called if a block state is replaced or it replaces another state
    async fn prepare(&self, _args: PrepareArgs<'_>) {}

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.state_id
    }

    async fn on_scheduled_tick(&self, _args: OnScheduledTickArgs<'_>) {}

    async fn on_state_replaced(&self, _args: OnStateReplacedArgs<'_>) {}

    /// Sides where redstone connects to
    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        false
    }

    /// Weak redstone power, aka. block that should be powered needs to be directly next to the source block
    async fn get_weak_redstone_power(&self, _args: GetRedstonePowerArgs<'_>) -> u8 {
        0
    }

    /// Strong redstone power. this can power a block that then gives power
    async fn get_strong_redstone_power(&self, _args: GetRedstonePowerArgs<'_>) -> u8 {
        0
    }

    async fn get_comparator_output(&self, _args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        None
    }
}

pub struct NormalUseArgs<'a> {
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
    pub player: &'a Player,
}

pub struct UseWithItemArgs<'a> {
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
    pub player: &'a Player,
    pub item_stack: &'a Arc<Mutex<ItemStack>>,
}

pub struct OnEntityCollisionArgs<'a> {
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub location: &'a BlockPos,
    pub entity: &'a dyn EntityBase,
}

pub struct ExplodeArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
}

pub struct OnSyncedBlockEventArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
    pub r#type: u8,
    pub data: u8,
}

pub struct OnPlaceArgs<'a> {
    pub server: &'a Server,
    pub world: &'a World,
    pub block: &'a Block,
    pub location: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: &'a Player,
    pub replacing: BlockIsReplacing,
    pub use_item_on: &'a SUseItemOn,
}

pub struct RandomTickArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
}

pub struct CanPlaceAtArgs<'a> {
    pub server: Option<&'a Server>,
    pub world: Option<&'a World>,
    pub block_accessor: &'a dyn BlockAccessor,
    pub block: &'a Block,
    pub location: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: Option<&'a Player>,
    pub use_item_on: Option<&'a SUseItemOn>,
}

pub struct CanUpdateAtArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub location: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: &'a Player,
    pub use_item_on: &'a SUseItemOn,
}

pub struct PlacedArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub old_state_id: BlockStateId,
    pub location: &'a BlockPos,
    pub notify: bool,
}

pub struct PlayerPlacedArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub location: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: &'a Player,
}

pub struct BrokenArgs<'a> {
    pub block: &'a Block,
    pub player: &'a Arc<Player>,
    pub location: &'a BlockPos,
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub state: &'a BlockState,
}

pub struct OnNeighborUpdateArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
    pub source_block: &'a Block,
    pub notify: bool,
}

pub struct PrepareArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub location: &'a BlockPos,
    pub flags: BlockFlags,
}

pub struct GetStateForNeighborUpdateArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub location: &'a BlockPos,
    pub direction: BlockDirection,
    pub neighbor_location: &'a BlockPos,
    pub neighbor_state_id: BlockStateId,
}

pub struct OnScheduledTickArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub location: &'a BlockPos,
}

pub struct OnStateReplacedArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub old_state_id: BlockStateId,
    pub location: &'a BlockPos,
    pub moved: bool,
}

pub struct EmitsRedstonePowerArgs<'a> {
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub direction: BlockDirection,
}

pub struct GetRedstonePowerArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub location: &'a BlockPos,
    pub direction: BlockDirection,
}

pub struct GetComparatorOutputArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub location: &'a BlockPos,
}
