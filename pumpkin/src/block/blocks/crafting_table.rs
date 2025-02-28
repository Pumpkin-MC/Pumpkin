use crate::block::container::ContainerBlock;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_inventory::{CraftingTable, OpenContainer};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;

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
    ) {
        self.open(block, player, _location, server).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open(block, player, _location, server).await;
        BlockActionResult::Consume
    }

    async fn broken(&self, _block: &Block, player: &Player, location: BlockPos, server: &Server) {
        self.destroy(location, server, player).await;
    }

    async fn close(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        container: &mut OpenContainer,
    ) {
        PumpkinBlock::close(self, block, player, location, server, container).await;

        // TODO: items should be re-added to player inventory or dropped depending on if they are in movement.
        // TODO: unique containers should be implemented as a separate stack internally (optimizes large player servers for example)
        // TODO: ephemeral containers (crafting tables) might need to be separate data structure than stored (ender chest)
    }
}

impl ContainerBlock<CraftingTable> for CraftingTableBlock {
    const UNIQUE: bool = true;
}
