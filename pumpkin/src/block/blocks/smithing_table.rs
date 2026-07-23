use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs};
use pumpkin_data::translation;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_inventory::smithing_screen_handler::SmithingScreenHandler;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;

#[pumpkin_block("minecraft:smithing_table")]
pub struct SmithingTableBlock;

impl BlockBehaviour for SmithingTableBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::InteractWithSmithingTable as i32,
                    1,
                )
                .await;
            args.player
                .open_handled_screen(&SmithingScreenFactory, Some(*args.position))
                .await;
            BlockActionResult::Success
        })
    }
}

struct SmithingScreenFactory;

impl ScreenHandlerFactory for SmithingScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler: SharedScreenHandler = Arc::new(Mutex::new(SmithingScreenHandler::new(
                sync_id,
                player_inventory,
            )));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        // Java title key is container.upgrade; bedrock uses smithing_table.
        TextComponent::translate_cross(
            translation::java::CONTAINER_UPGRADE,
            translation::bedrock::CONTAINER_SMITHING_TABLE,
            &[],
        )
    }
}
