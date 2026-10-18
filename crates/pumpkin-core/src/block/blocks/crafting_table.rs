use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, GetScreenHandlerFactoryArgs, NormalUseArgs};

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
        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::InteractWithCraftingTable as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }

        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let recipe_manager = args.server.recipe_manager.clone();
        Some(Box::new(CraftingTableScreenFactory(recipe_manager)))
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
