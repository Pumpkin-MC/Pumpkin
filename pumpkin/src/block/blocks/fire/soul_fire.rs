use std::sync::Arc;

use pumpkin_data::tag::Tagable;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

use super::FireBlockBase;

#[pumpkin_block("minecraft:soul_fire")]
pub struct SoulFireBlock;

impl SoulFireBlock {
    pub fn is_soul_base(block: &Block) -> bool {
        block
            .is_tagged_with("minecraft:soul_fire_base_blocks")
            .unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SoulFireBlock {
    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        FireBlockBase::placed(
            &FireBlockBase,
            world,
            block,
            state_id,
            pos,
            old_state_id,
            notify,
        )
        .await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state: BlockStateId,
        block_pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if self.can_place_at(world, block_pos, direction).await {
            return state;
        }
        Block::AIR.default_state_id
    }

    async fn can_place_at(
        &self,
        world: &World,
        block_pos: &BlockPos,
        _face: BlockDirection,
    ) -> bool {
        Self::is_soul_base(&world.get_block(&block_pos.down()).await.unwrap())
    }

    async fn broken(
        &self,
        block: &Block,
        player: &Player,
        position: BlockPos,
        server: &Server,
        world: Arc<World>,
        state: BlockState,
    ) {
        FireBlockBase::broken(
            &FireBlockBase,
            block,
            player,
            position,
            server,
            world,
            state,
        )
        .await;
    }
}
