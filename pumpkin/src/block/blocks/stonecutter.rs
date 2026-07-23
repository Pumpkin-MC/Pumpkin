use std::sync::Arc;

use crate::block::{
    BlockBehaviour, BlockFuture, NormalUseArgs, OnPlaceArgs, registry::BlockActionResult,
};
use pumpkin_data::{
    BlockStateId,
    block_properties::{BlockProperties, WallTorchLikeProperties},
    translation,
};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_inventory::stonecutter_screen_handler::StonecutterScreenHandler;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;

#[pumpkin_block("minecraft:stonecutter")]
pub struct StonecutterBlock;

impl BlockBehaviour for StonecutterBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        // Horizontal facing (packed as WallTorchLikeProperties for this block id).
        Box::pin(async move {
            let mut props = WallTorchLikeProperties::default(args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.to_state_id(args.block)
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::InteractWithStonecutter as i32,
                    1,
                )
                .await;
            args.player
                .open_handled_screen(&StonecutterScreenFactory, Some(*args.position))
                .await;

            BlockActionResult::Success
        })
    }
}

struct StonecutterScreenFactory;

impl ScreenHandlerFactory for StonecutterScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler: SharedScreenHandler = Arc::new(Mutex::new(StonecutterScreenHandler::new(
                sync_id,
                player_inventory,
            )));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::CONTAINER_STONECUTTER,
            translation::bedrock::CONTAINER_STONECUTTER,
            &[],
        )
    }
}
