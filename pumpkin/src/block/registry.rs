use crate::block::pumpkin_block::{
    BlockHitResult, BlockMetadata, OnEntityCollisionArgs, PumpkinBlock,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties;
use pumpkin_data::fluid;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::{BlockAccessor, BlockFlags, BlockRegistryExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::BlockIsReplacing;
use super::pumpkin_block::{
    BrokenArgs, CanPlaceAtArgs, CanUpdateAtArgs, EmitsRedstonePowerArgs, ExplodeArgs,
    GetRedstonePowerArgs, GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnStateReplacedArgs, OnSyncedBlockEventArgs, PlacedArgs, PlayerPlacedArgs,
    PrepareArgs, UseWithItemArgs,
};
use super::pumpkin_fluid::PumpkinFluid;

// ActionResult.java
pub enum BlockActionResult {
    /// Action was successful | Same as SUCCESS in vanilla
    Success,
    /// Action was successful and we should swing the hand for the server | Same as `SUCCESS_SERVER` in vanilla
    SuccessServer,
    /// Block other actions from being executed | Same as CONSUME in vanilla
    Consume,
    /// Allow other actions to be executed, but indicate it failed | Same as FAIL in vanilla
    Fail,
    /// Allow other actions to be executed | Same as PASS in vanilla
    Pass,
    /// Use default action for the block: `normal_use` | Same as `PASS_TO_DEFAULT_BLOCK_ACTION` in vanilla
    PassToDefaultBlockAction,
}

impl BlockActionResult {
    #[must_use]
    pub fn consumes_action(&self) -> bool {
        matches!(self, Self::Consume | Self::Success | Self::SuccessServer)
    }
}

#[derive(Default)]
pub struct BlockRegistry {
    blocks: HashMap<&'static Block, Arc<dyn PumpkinBlock>>,
    fluids: HashMap<&'static Fluid, Arc<dyn PumpkinFluid>>,
}

#[async_trait]
impl BlockRegistryExt for BlockRegistry {
    fn can_place_at(
        &self,
        block: &pumpkin_data::Block,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        face: BlockDirection,
    ) -> bool {
        futures::executor::block_on(async move {
            self.can_place_at(
                None,
                None,
                block_accessor,
                None,
                block,
                block_pos,
                face,
                None,
            )
            .await
        })
    }
}

impl BlockRegistry {
    pub fn register<T: PumpkinBlock + BlockMetadata + 'static>(&mut self, block: T) {
        let names = block.names();
        let val = Arc::new(block);
        self.blocks.reserve(names.len());
        for i in names {
            self.blocks.insert(
                block_properties::get_block(i.as_str()).unwrap(),
                val.clone(),
            );
        }
    }

    pub fn register_fluid<T: PumpkinFluid + BlockMetadata + 'static>(&mut self, fluid: T) {
        let names = fluid.names();
        let val = Arc::new(fluid);
        self.fluids.reserve(names.len());
        for i in names {
            self.fluids
                .insert(fluid::get_fluid(i.as_str()).unwrap(), val.clone());
        }
    }

    pub async fn on_synced_block_event(
        &self,
        block: &Block,
        world: &Arc<World>,
        position: &BlockPos,
        r#type: u8,
        data: u8,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .on_synced_block_event(OnSyncedBlockEventArgs {
                    world,
                    block,
                    position,
                    r#type,
                    data,
                })
                .await;
        }
        false
    }

    pub async fn on_entity_collision(
        &self,
        block: &Block,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        position: &BlockPos,
        state: &BlockState,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_entity_collision(OnEntityCollisionArgs {
                    server,
                    world,
                    block,
                    state,
                    position,
                    entity,
                })
                .await;
        }
    }

    pub async fn on_entity_collision_fluid(&self, fluid: &Fluid, entity: &dyn EntityBase) {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            pumpkin_fluid.on_entity_collision(entity).await;
        }
    }

    pub async fn on_use(
        &self,
        block: &Block,
        player: &Player,
        position: &BlockPos,
        hit: &BlockHitResult<'_>,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .normal_use(NormalUseArgs {
                    server,
                    world,
                    block,
                    position,
                    player,
                    hit,
                })
                .await;
        }
        BlockActionResult::Pass
    }

    pub async fn explode(&self, block: &Block, world: &Arc<World>, position: &BlockPos) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .explode(ExplodeArgs {
                    world,
                    block,
                    position,
                })
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        position: &BlockPos,
        hit: &BlockHitResult<'_>,
        item_stack: &Arc<Mutex<ItemStack>>,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .use_with_item(UseWithItemArgs {
                    server,
                    world,
                    block,
                    position,
                    player,
                    hit,
                    item_stack,
                })
                .await;
        }
        BlockActionResult::Pass
    }

    pub async fn use_with_item_fluid(
        &self,
        fluid: &Fluid,
        player: &Player,
        position: BlockPos,
        item: &Item,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            return pumpkin_fluid
                .use_with_item(fluid, player, position, item, server, world)
                .await;
        }
        BlockActionResult::Pass
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn can_place_at(
        &self,
        server: Option<&Server>,
        world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        player: Option<&Player>,
        block: &Block,
        position: &BlockPos,
        direction: BlockDirection,
        use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .can_place_at(CanPlaceAtArgs {
                    server,
                    world,
                    block_accessor,
                    block,
                    position,
                    direction,
                    player,
                    use_item_on,
                })
                .await;
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn can_update_at(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        use_item_on: &SUseItemOn,
        player: &Player,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .can_update_at(CanUpdateAtArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    player,
                    use_item_on,
                })
                .await;
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn on_place(
        &self,
        server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        position: &BlockPos,
        direction: BlockDirection,
        replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .on_place(OnPlaceArgs {
                    server,
                    world,
                    block,
                    position,
                    direction,
                    player,
                    replacing,
                    use_item_on,
                })
                .await;
        }
        block.default_state.id
    }

    pub async fn player_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: u16,
        position: &BlockPos,
        direction: BlockDirection,
        player: &Player,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .player_placed(PlayerPlacedArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    player,
                })
                .await;
        }
    }

    pub async fn on_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .placed(PlacedArgs {
                    world,
                    block,
                    state_id,
                    old_state_id,
                    position,
                    notify,
                })
                .await;
        }
    }

    pub async fn on_placed_fluid(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        state_id: BlockStateId,
        position: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            pumpkin_fluid
                .placed(world, fluid, state_id, position, old_state_id, notify)
                .await;
        }
    }

    pub async fn broken(
        &self,
        world: &Arc<World>,
        block: &Block,
        player: &Arc<Player>,
        position: &BlockPos,
        server: &Server,
        state: &BlockState,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .broken(BrokenArgs {
                    block,
                    player,
                    position,
                    server,
                    world,
                    state,
                })
                .await;
        }
    }

    pub async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        position: &BlockPos,
        old_state_id: BlockStateId,
        moved: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_state_replaced(OnStateReplacedArgs {
                    world,
                    block,
                    old_state_id,
                    position,
                    moved,
                })
                .await;
        }
    }

    /// Updates state of all neighbors of the block
    pub async fn post_process_state(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
        flags: BlockFlags,
    ) {
        let state = world.get_block_state(position).await;
        for direction in BlockDirection::all() {
            let neighbor_pos = position.offset(direction.to_offset());
            let neighbor_state = world.get_block_state(&neighbor_pos).await;
            let pumpkin_block = self.get_pumpkin_block(block);
            if let Some(pumpkin_block) = pumpkin_block {
                let new_state = pumpkin_block
                    .get_state_for_neighbor_update(GetStateForNeighborUpdateArgs {
                        world,
                        block,
                        state_id: state.id,
                        position,
                        direction: direction.opposite(),
                        neighbor_position: &neighbor_pos,
                        neighbor_state_id: neighbor_state.id,
                    })
                    .await;
                world.set_block_state(&neighbor_pos, new_state, flags).await;
            }
        }
    }

    pub async fn prepare(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
        flags: BlockFlags,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .prepare(PrepareArgs {
                    world,
                    block,
                    state_id,
                    position,
                    flags,
                })
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_state_for_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        neighbor_location: &BlockPos,
        neighbor_state_id: BlockStateId,
    ) -> BlockStateId {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_state_for_neighbor_update(GetStateForNeighborUpdateArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    neighbor_position: neighbor_location,
                    neighbor_state_id,
                })
                .await;
        }
        state_id
    }

    pub async fn update_neighbors(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        _block: &Block,
        flags: BlockFlags,
    ) {
        for direction in BlockDirection::abstract_block_update_order() {
            let pos = position.offset(direction.to_offset());

            Box::pin(world.replace_with_state_for_neighbor_update(
                &pos,
                direction.opposite(),
                flags,
            ))
            .await;
        }
    }

    pub async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        position: &BlockPos,
        source_block: &Block,
        notify: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_neighbor_update(OnNeighborUpdateArgs {
                    world,
                    block,
                    position,
                    source_block,
                    notify,
                })
                .await;
        }
    }

    #[must_use]
    pub fn get_pumpkin_block(&self, block: &Block) -> Option<&Arc<dyn PumpkinBlock>> {
        self.blocks.get(block)
    }

    #[must_use]
    pub fn get_pumpkin_fluid(&self, fluid: &Fluid) -> Option<&Arc<dyn PumpkinFluid>> {
        self.fluids.get(fluid)
    }

    pub async fn emits_redstone_power(
        &self,
        block: &Block,
        state: &BlockState,
        direction: BlockDirection,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .emits_redstone_power(EmitsRedstonePowerArgs {
                    block,
                    state,
                    direction,
                })
                .await;
        }
        false
    }

    pub async fn get_weak_redstone_power(
        &self,
        block: &Block,
        world: &World,
        position: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_weak_redstone_power(GetRedstonePowerArgs {
                    world,
                    block,
                    state,
                    position,
                    direction,
                })
                .await;
        }
        0
    }

    pub async fn get_strong_redstone_power(
        &self,
        block: &Block,
        world: &World,
        position: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_strong_redstone_power(GetRedstonePowerArgs {
                    world,
                    block,
                    state,
                    position,
                    direction,
                })
                .await;
        }
        0
    }
}
