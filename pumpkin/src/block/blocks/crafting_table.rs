use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::{block::pumpkin_block::PumpkinBlock, world::World};
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockState};
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;

#[pumpkin_block("minecraft:crafting_table")]
pub struct CraftingTableBlock;

#[async_trait]
impl PumpkinBlock for CraftingTableBlock {
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
