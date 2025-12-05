use std::sync::Arc;

use crate::block::{BlockFuture, OnPlaceArgs, PlacedArgs};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};

use pumpkin_data::block_properties::{BarrelLikeProperties, BlockProperties};
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::barrel::BarrelBlockEntity;
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

struct BarrelScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for BarrelScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone()).await;
            let concrete_arc = Arc::new(Mutex::new(handler));

            Some(concrete_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.barrel", &[])
    }
}

#[pumpkin_block("minecraft:barrel")]
pub struct BarrelBlock;

impl BlockBehaviour for BarrelBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = BarrelLikeProperties::default(args.block);
            props.facing = args.player.living_entity.entity.get_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&BarrelScreenFactory(inventory))
                    .await;
            }

            BlockActionResult::Success
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let barrel_block_entity = BarrelBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(barrel_block_entity))
                .await;
        })
    }
}
