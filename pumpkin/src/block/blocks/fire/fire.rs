use std::sync::Arc;

use pumpkin_world::chunk::TickPriority;
use rand::Rng;

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

#[pumpkin_block("minecraft:fire")]
pub struct FireBlock;

impl FireBlock {
    pub fn get_fire_tick_delay() -> i32 {
        30 + rand::thread_rng().gen_range(0..10)
    }
}

#[async_trait]
impl PumpkinBlock for FireBlock {
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
        world
            .schedule_block_tick(
                block,
                *pos,
                Self::get_fire_tick_delay() as u16,
                TickPriority::Normal,
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
        // TODO: add can_place_at
        if self.can_place_at(world, block_pos, direction).await {
            return state;
        }
        Block::AIR.default_state_id
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
