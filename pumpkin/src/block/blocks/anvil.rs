use crate::block::blocks::falling::FallingBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, NormalUseArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};

use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::translation;
use pumpkin_inventory::anvil::AnvilScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::inventory::SimpleInventory;
use std::sync::Arc;
use tokio::sync::Mutex;

#[pumpkin_block_from_tag("minecraft:anvil")]
pub struct AnvilBlock;

impl BlockBehaviour for AnvilBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            args.player
                .open_handled_screen(&AnvilScreenFactory, Some(*args.position))
                .await;

            BlockActionResult::Success
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FallingBlock::placed(&FallingBlock, args).await;
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let dir = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .rotate_clockwise();

            let mut props = WallTorchLikeProperties::default(args.block);

            props.facing = dir;
            props.to_state_id(args.block)
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FallingBlock::on_scheduled_tick(&FallingBlock, args).await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(
            async move { FallingBlock::get_state_for_neighbor_update(&FallingBlock, args).await },
        )
    }
}

struct AnvilScreenFactory;

impl ScreenHandlerFactory for AnvilScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let inventory = Arc::new(SimpleInventory::new(3));
            let handler = AnvilScreenHandler::new(sync_id, player_inventory, inventory);
            let concrete_arc = Arc::new(Mutex::new(handler));

            Some(concrete_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::CONTAINER_REPAIR,
            translation::bedrock::CONTAINER_REPAIR,
            &[],
        )
    }
}
