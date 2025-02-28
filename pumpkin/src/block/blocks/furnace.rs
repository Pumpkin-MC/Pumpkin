use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_inventory::Furnace;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;

use crate::block::container::ContainerBlock;
use crate::{block::pumpkin_block::PumpkinBlock, server::Server};

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        self.open(block, player, location, server).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open(block, player, location, server).await;
        BlockActionResult::Consume
    }

    async fn broken(&self, _block: &Block, player: &Player, location: BlockPos, server: &Server) {
        self.destroy(location, server, player).await;
    }
}

impl ContainerBlock<Furnace> for FurnaceBlock {
    const UNIQUE: bool = false;
}
