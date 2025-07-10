use std::sync::Arc;

use crate::block::pumpkin_block::{OnPlaceArgs, OnStateReplacedArgs, PlacedArgs};
use crate::block::{
    pumpkin_block::{NormalUseArgs, PumpkinBlock},
    registry::BlockActionResult,
};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BarrelLikeProperties, BlockProperties};
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::barrel::BarrelBlockEntity;
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

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
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if let Some(inventory) = block_entity.get_inventory() {
                args.player
                    .open_handled_screen(&BarrelScreenFactory(inventory))
                    .await;
            }
        }

        BlockActionResult::Success
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BarrelLikeProperties::default(args.block);
        props.facing = args.player.living_entity.entity.get_facing().opposite();
        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let barrel_block_entity = BarrelBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(barrel_block_entity))
            .await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world.remove_block_entity(args.position).await;
    }
}
