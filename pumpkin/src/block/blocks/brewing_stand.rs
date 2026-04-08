use std::sync::Arc;

use crate::block::{BlockFuture, PlacedArgs};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};

use pumpkin_data::translation;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{BoxFuture, ScreenHandlerFactory, SharedScreenHandler};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::block::entities::brewing_stand::BrewingStandBlockEntity;
use pumpkin_world::inventory::Inventory;

struct BrewingScreenFactory(
    Arc<dyn Inventory>,
    Arc<dyn pumpkin_world::block::entities::PropertyDelegate>,
);

impl ScreenHandlerFactory for BrewingScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn pumpkin_inventory::screen_handler::InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        let inventory = self.0.clone();
        let property_delegate = self.1.clone();
        let pi = player_inventory.clone();
        Box::pin(async move {
            // Delegate to pumpkin-inventory brewing creator
            pumpkin_inventory::brewing::create_brewing(sync_id, pi, inventory, property_delegate)
                .await
                .map(|handler| Arc::new(tokio::sync::Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate(translation::CONTAINER_BREWING, &[])
    }
}

#[pumpkin_block("minecraft:brewing_stand")]
pub struct BrewingStandBlock;

impl BlockBehaviour for BrewingStandBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.clone().get_inventory()
                && let Some(pd) = block_entity.clone().to_property_delegate()
            {
                args.player
                    .open_handled_screen(&BrewingScreenFactory(inventory, pd), Some(*args.position))
                    .await;
            }

            BlockActionResult::Success
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let be = BrewingStandBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(be)).await;
        })
    }
}
