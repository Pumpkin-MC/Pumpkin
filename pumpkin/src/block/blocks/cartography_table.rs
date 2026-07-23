use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs};
use pumpkin_data::translation;
use pumpkin_inventory::cartography_screen_handler::CartographyScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
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
                .open_handled_screen(&CartographyScreenFactory, Some(*args.position))
                .await;
            BlockActionResult::Success
        })
    }
}

struct CartographyScreenFactory;

impl ScreenHandlerFactory for CartographyScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler: SharedScreenHandler = Arc::new(Mutex::new(CartographyScreenHandler::new(
                sync_id,
                player_inventory,
            )));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        // No dedicated bedrock container key; reuse Java key for both.
        TextComponent::translate_cross(
            translation::java::CONTAINER_CARTOGRAPHY_TABLE,
            translation::java::CONTAINER_CARTOGRAPHY_TABLE,
            &[],
        )
    }
}
