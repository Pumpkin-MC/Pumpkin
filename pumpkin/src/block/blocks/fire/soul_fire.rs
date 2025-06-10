use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

use super::FireBlockBase;

#[pumpkin_block("minecraft:soul_fire")]
pub struct SoulFireBlock;

impl SoulFireBlock {
    #[must_use]
    pub fn is_soul_base(block: &Block) -> bool {
        block
            .is_tagged_with("minecraft:soul_fire_base_blocks")
            .unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SoulFireBlock {
    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        _block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !Self::is_soul_base(&world.get_block(&block_pos.down()).await) {
            return Block::AIR.default_state_id;
        }

        state_id
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        FireBlockBase::can_place_at(block_accessor, block_pos).await
            && Self::is_soul_base(&block_accessor.get_block(&block_pos.down()).await)
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Arc<Player>,
        block_pos: BlockPos,
        _server: &Server,
        world: Arc<World>,
        _state: BlockState,
    ) {
        FireBlockBase::broken(world, block_pos).await;
    }
}
