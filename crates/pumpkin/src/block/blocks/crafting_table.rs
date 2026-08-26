use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs};

use pumpkin_data::translation;
use pumpkin_inventory::crafting::crafting_screen_handler::CraftingTableScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::Mutex;

#[pumpkin_block("minecraft:crafting_table")]
pub struct CraftingTableBlock;

impl BlockBehaviour for CraftingTableBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::InteractWithCraftingTable as i32,
            1,
        );
        let recipe_manager = args.server.recipe_manager.clone();
        args.player.open_handled_screen(
            &CraftingTableScreenFactory(recipe_manager),
            Some(*args.position),
        );

        BlockActionResult::Success
    }
}

struct CraftingTableScreenFactory(Arc<crate::server::RecipeManager>);

impl ScreenHandlerFactory for CraftingTableScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler =
            CraftingTableScreenHandler::new(sync_id, player_inventory, Some(self.0.clone()));
        let concrete_arc = Arc::new(Mutex::new(handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_CRAFTING,
            translation::bedrock::CONTAINER_CRAFTING
        )
    }
}
