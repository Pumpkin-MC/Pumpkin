use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::block::entities::barrel::BarrelBlockEntity;
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

use crate::block::pumpkin_block::{OnStateReplacedArgs, PlacedArgs, UseWithItemArgs};
use crate::block::{
    pumpkin_block::{NormalUseArgs, PumpkinBlock},
    registry::BlockActionResult,
};

struct BarrelScreenFactory(Arc<dyn Inventory>);

#[async_trait]
impl ScreenHandlerFactory for BarrelScreenFactory {
    async fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn ScreenHandler>>> {
        #[allow(clippy::option_if_let_else)]
        Some(Arc::new(Mutex::new(create_generic_9x3(
            sync_id,
            player_inventory,
            self.0.clone(),
        ))))
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate("container.barrel", &[])
    }
}

#[pumpkin_block("minecraft:barrel")]
pub struct BarrelBlock;

#[async_trait]
impl PumpkinBlock for BarrelBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        if let Some(block_entity) = args.world.get_block_entity(args.location).await {
            if let Some(inventory) = block_entity.1.get_inventory() {
                args.player
                    .open_handled_screen(&BarrelScreenFactory(inventory))
                    .await;
            }
        }
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.location).await {
            if let Some(inventory) = block_entity.1.get_inventory() {
                args.player
                    .open_handled_screen(&BarrelScreenFactory(inventory))
                    .await;
            }
        }
        BlockActionResult::Consume
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let barrel_block_entity = BarrelBlockEntity::new(*args.location);
        args.world
            .add_block_entity(Arc::new(barrel_block_entity))
            .await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.location).await;
    }
}
