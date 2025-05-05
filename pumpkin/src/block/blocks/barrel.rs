use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockState};
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;
use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
};


#[pumpkin_block("minecraft:barrel")]
pub struct BarrelBlock;

#[async_trait]
impl PumpkinBlock for BarrelBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        server: &Server,
        _world: &Arc<World>,
    ) {
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
        BlockActionResult::Consume
    }

    async fn broken(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        _world: Arc<World>,
        _state: BlockState,
    ) {
    }
}
