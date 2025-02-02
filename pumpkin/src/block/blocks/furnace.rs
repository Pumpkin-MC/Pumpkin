use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::Furnace;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use pumpkin_world::item::registry::Item;

use crate::{block::pumpkin_block::PumpkinBlock, server::Server};

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        server: &Server,
    ) {
        self.open_furnace_screen(block, player, _location, server)
            .await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open_furnace_screen(block, player, _location, server)
            .await;
        BlockActionResult::Consume
    }

    async fn broken(&self, block: &Block, player: &Player, location: BlockPos, server: &Server) {
        super::standard_on_broken_with_container(block, player, location, server).await;
    }
}

impl FurnaceBlock {
    pub async fn open_furnace_screen(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        super::standard_open_container::<Furnace>(
            block,
            player,
            location,
            server,
            WindowType::Furnace,
        )
        .await;
    }
}
