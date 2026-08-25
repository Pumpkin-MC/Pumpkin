use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs};

use pumpkin_data::translation;
use pumpkin_inventory::cartography_table_screen_handler::CartographyTableScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use tokio::sync::Mutex;

#[pumpkin_block("minecraft:cartography_table")]
pub struct CartographyTableBlock;

impl BlockBehaviour for CartographyTableBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::InteractWithCartographyTable as i32,
                    1,
                )
                .await;
            args.player
                .open_handled_screen(&CartographyTableScreenFactory, Some(*args.position))
                .await;

            BlockActionResult::Success
        })
    }
}

struct CartographyTableScreenFactory;

impl ScreenHandlerFactory for CartographyTableScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler: SharedScreenHandler = Arc::new(Mutex::new(
                CartographyTableScreenHandler::new(sync_id, player_inventory),
            ));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate(translation::java::CONTAINER_CARTOGRAPHY_TABLE, [])
    }
}
