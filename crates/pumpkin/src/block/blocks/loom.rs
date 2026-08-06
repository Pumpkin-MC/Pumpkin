use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_inventory::loom_screen_handler::LoomScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture as ScreenHandlerBoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs, OnPlaceArgs};

#[pumpkin_block("minecraft:loom")]
pub struct LoomBlock;

impl BlockBehaviour for LoomBlock {
    // getStateForPlacement (LoomBlock.java:52-55): faces the placing player, same as
    // stonecutter.rs's on_place.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
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

    // useWithoutItem (LoomBlock.java:33-43): opens the banner-pattern menu and awards the
    // interaction stat.
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::InteractWithLoom as i32,
                    1,
                )
                .await;
            args.player
                .open_handled_screen(&LoomScreenFactory, Some(*args.position))
                .await;

            BlockActionResult::Success
        })
    }
}

struct LoomScreenFactory;

impl ScreenHandlerFactory for LoomScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerBoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler: SharedScreenHandler = Arc::new(Mutex::new(LoomScreenHandler::new(
                sync_id,
                player_inventory,
            )));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate_cross(
            pumpkin_data::translation::java::CONTAINER_LOOM,
            pumpkin_data::translation::bedrock::CONTAINER_LOOM,
            &[],
        )
    }
}
