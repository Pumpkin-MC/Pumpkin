use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_inventory::screen_handler::ScreenHandlerFactory;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::{block::entities::furnace::FurnaceBlockEntity, inventory::Inventory};
use tokio::sync::Mutex;

use crate::{block::pumpkin_block::PumpkinBlock, item::fuel_registry::FuelRegistry};

use super::furnace_screen_handler::FurnaceScreenHandler;

struct FurnaceScreenFactory {
    inventory: Arc<dyn Inventory>,
    fuel_registery: Arc<FuelRegistry>,
}

impl FurnaceScreenFactory {
    fn new(inventory: Arc<dyn Inventory>, fuel_registery: Arc<FuelRegistry>) -> Self {
        Self {
            inventory,
            fuel_registery,
        }
    }
}

impl ScreenHandlerFactory for FurnaceScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        _player: &dyn pumpkin_inventory::screen_handler::InventoryPlayer,
    ) -> Option<Arc<tokio::sync::Mutex<dyn pumpkin_inventory::screen_handler::ScreenHandler>>> {
        Some(Arc::new(Mutex::new(FurnaceScreenHandler::new(
            sync_id,
            player_inventory,
            self.inventory.clone(),
            self.fuel_registery.clone(),
        ))))
    }

    fn get_display_name(&self) -> pumpkin_util::text::TextComponent {
        TextComponent::translate("container.furnace", &[])
    }
}

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn normal_use(
        &self,
        _block: &pumpkin_data::Block,
        player: &crate::entity::player::Player,
        location: pumpkin_util::math::position::BlockPos,
        server: &crate::server::Server,
        world: &std::sync::Arc<crate::world::World>,
    ) {
        if let Some((_, block_entity)) = world.get_block_entity(&location).await {
            if let Some(inventory) = block_entity.get_inventory() {
                let fuel_registery = server.fuel_registry.clone();
                let furnace_screen_factory = FurnaceScreenFactory::new(inventory, fuel_registery);
                player.open_handled_screen(&furnace_screen_factory).await;
            }
        }
    }

    async fn use_with_item(
        &self,
        _block: &pumpkin_data::Block,
        player: &crate::entity::player::Player,
        location: pumpkin_util::math::position::BlockPos,
        _item: &pumpkin_data::item::Item,
        server: &crate::server::Server,
        world: &std::sync::Arc<crate::world::World>,
    ) -> crate::block::registry::BlockActionResult {
        if let Some((_, block_entity)) = world.get_block_entity(&location).await {
            if let Some(inventory) = block_entity.get_inventory() {
                let fuel_registery = server.fuel_registry.clone();
                let furnace_screen_factory = FurnaceScreenFactory::new(inventory, fuel_registery);
                player.open_handled_screen(&furnace_screen_factory).await;
            }
        }
        crate::block::registry::BlockActionResult::Consume
    }
    async fn on_synced_block_event(
        &self,
        _block: &pumpkin_data::Block,
        _world: &std::sync::Arc<crate::world::World>,
        _pos: &pumpkin_util::math::position::BlockPos,
        _type: u8,
        _data: u8,
    ) -> bool {
        false
    }

    async fn placed(
        &self,
        _world: &std::sync::Arc<crate::world::World>,
        _block: &pumpkin_data::Block,
        _state_id: pumpkin_world::BlockStateId,
        _pos: &pumpkin_util::math::position::BlockPos,
        _old_state_id: pumpkin_world::BlockStateId,
        _notify: bool,
    ) {
        let furnace_block_entity = FurnaceBlockEntity::new(*_pos);
        _world
            .add_block_entity(Arc::new(furnace_block_entity))
            .await;
    }

    async fn on_state_replaced(
        &self,
        world: &std::sync::Arc<crate::world::World>,
        _block: &pumpkin_data::Block,
        location: pumpkin_util::math::position::BlockPos,
        _old_state_id: pumpkin_world::BlockStateId,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }
}

