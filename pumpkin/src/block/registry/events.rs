use super::{BlockActionResult, BlockRegistry};
use crate::block::fluid::FluidBehaviour;
use crate::block::{
    BlockBehaviour, BlockHitResult, BrokenArgs, ExplodeArgs, NormalUseArgs, OnEntityCollisionArgs,
    OnEntityStepArgs, OnLandedUponArgs, OnStateReplacedArgs, OnSyncedBlockEventArgs,
    UpdateEntityMovementAfterFallOnArgs, UseWithItemArgs, stop_vertical_movement_after_fall,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;
use tokio::sync::Mutex;

impl BlockRegistry {
    pub async fn on_synced_block_event(
        &self,
        block: &Block,
        world: &Arc<World>,
        position: &BlockPos,
        r#type: u8,
        data: u8,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block.id);
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
        let pumpkin_block = self.get_pumpkin_block(block.id);
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

    pub async fn on_entity_step(
        &self,
        block: &Block,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        position: &BlockPos,
        state: &BlockState,
        below_supporting_block: bool,
    ) {
        if let Some(pumpkin_block) = self.get_pumpkin_block(block.id) {
            pumpkin_block
                .on_entity_step(OnEntityStepArgs {
                    world,
                    block,
                    state,
                    position,
                    entity,
                    below_supporting_block,
                })
                .await;
        }
    }

    pub async fn on_entity_collision_fluid(&self, fluid: &Fluid, entity: &dyn EntityBase) {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid.id);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            pumpkin_fluid.on_entity_collision(entity).await;
        }
    }

    pub async fn on_use(
        &self,
        block: &Block,
        player: &Arc<Player>,
        position: &BlockPos,
        hit: &BlockHitResult<'_>,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block.id);
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
        let pumpkin_block = self.get_pumpkin_block(block.id);
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

    #[expect(clippy::too_many_arguments)]
    pub async fn use_with_item(
        &self,
        block: &Block,
        player: &Arc<Player>,
        position: &BlockPos,
        hit: &BlockHitResult<'_>,
        item_stack: &Arc<Mutex<ItemStack>>,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block.id);
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
        player: &Arc<Player>,
        position: BlockPos,
        item: &Item,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid.id);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            return pumpkin_fluid
                .use_with_item(fluid, player, position, item, server, world)
                .await;
        }
        BlockActionResult::Pass
    }

    pub async fn on_landed_upon(
        &self,
        block: &Block,
        world: &Arc<World>,
        fall_distance: f32,
        entity: &dyn EntityBase,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_landed_upon(OnLandedUponArgs {
                    world,
                    fall_distance,
                    entity,
                })
                .await;
        }
    }

    pub async fn update_entity_movement_after_fall_on(
        &self,
        block: &Block,
        entity: &dyn EntityBase,
    ) {
        if let Some(pumpkin_block) = self.get_pumpkin_block(block.id) {
            pumpkin_block
                .update_entity_movement_after_fall_on(UpdateEntityMovementAfterFallOnArgs {
                    entity,
                })
                .await;
        } else {
            stop_vertical_movement_after_fall(entity);
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
        let pumpkin_block = self.get_pumpkin_block(block.id);
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
        let pumpkin_block = self.get_pumpkin_block(block.id);
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
}
