use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, GetScreenHandlerFactoryArgs, NormalUseArgs};

use pumpkin_data::translation;
use pumpkin_inventory::cartography_table_screen_handler::CartographyTableScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::Mutex;

#[pumpkin_block("minecraft:cartography_table")]
pub struct CartographyTableBlock;

impl BlockBehaviour for CartographyTableBlock {
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
                pumpkin_data::statistic::CustomStatistic::InteractWithCartographyTable as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }

        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        _args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        Some(Box::new(CartographyTableScreenFactory))
    }
}

struct CartographyTableScreenFactory;

impl ScreenHandlerFactory for CartographyTableScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler: SharedScreenHandler = Arc::new(Mutex::new(
            CartographyTableScreenHandler::new(sync_id, player_inventory),
        ));
        Some(handler)
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate(translation::java::CONTAINER_CARTOGRAPHY_TABLE, [])
    }
}
