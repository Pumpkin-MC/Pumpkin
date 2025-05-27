use std::sync::Arc;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:dirt_path")]
pub struct DirtPathBlock;

#[async_trait]
impl PumpkinBlock for DirtPathBlock {
    async fn on_scheduled_tick(&self, world: &Arc<World>, _block: &Block, pos: &BlockPos) {
        // TODO: push up entities
        world
            .set_block_state(pos, Block::DIRT.default_state_id, BlockFlags::NOTIFY_ALL)
            .await;
    }

    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        _player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        if !can_place_at(world, block_pos).await {
            return Block::DIRT.default_state_id;
        }

        block.default_state_id
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == BlockDirection::Up && !can_place_at(world, pos).await {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }
        state
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(world.unwrap(), block_pos).await
    }
}

async fn can_place_at(world: &World, block_pos: &BlockPos) -> bool {
    let state = world.get_block_state(&block_pos.up()).await;
    !state.is_solid() // TODO: add fence gate block
}
