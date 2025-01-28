use crate::block::block_manager::BlockActionResult;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::{CraftingTable, OpenContainer};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{block::block_registry::Block, item::item_registry::Item};

#[pumpkin_block("minecraft:crafting_table")]
pub struct CraftingTableBlock;

#[async_trait]
impl PumpkinBlock for CraftingTableBlock {
    async fn on_use<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        server: &Server,
    ) {
        self.open_crafting_screen(block, player, _location, server)
            .await;
    }

    async fn on_use_with_item<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open_crafting_screen(block, player, _location, server)
            .await;
        BlockActionResult::Consume
    }

    async fn on_broken<'a>(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        super::standard_on_broken_with_container(block, player, location, server).await;
    }

    async fn on_close<'a>(
        &self,
        _block: &Block,
        player: &Player,
        _location: BlockPos,
        _server: &Server,
        container: &mut OpenContainer,
    ) {
        let entity_id = player.entity_id();
        for player_id in container.all_player_ids() {
            if entity_id == player_id {
                container.clear_all_slots().await;
            }
        }

        container.remove_player(entity_id);

        // TODO: items should be re-added to player inventory or dropped depending on if they are in movement.
        // TODO: unique containers should be implemented as a separate stack internally (optimizes large player servers for example)
        // TODO: ephemeral containers (crafting tables) might need to be separate data structure than stored (ender chest)
    }
}

impl CraftingTableBlock {
    pub async fn open_crafting_screen(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        super::standard_open_container_unique::<CraftingTable>(
            block,
            player,
            location,
            server,
            WindowType::Crafting,
        )
        .await;
    }
}
