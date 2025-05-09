use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerFactory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::barrel::BarrelBlockEntity;
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

use crate::world::World;
use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
};

struct BarrelScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for BarrelScreenFactory {
    fn create_screen_handler(
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
        TextComponent::translate("container.barrel", vec![])
    }
}

#[pumpkin_block("minecraft:barrel")]
pub struct BarrelBlock;

#[async_trait]
impl PumpkinBlock for BarrelBlock {
    async fn normal_use(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        if let Some(block_entity) = world.get_block_entity(&location).await {
            if let Some(inventory) = block_entity.get_inventory() {
                player
                    .open_handled_screen(&BarrelScreenFactory(inventory))
                    .await;
            }
        }
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        if let Some(block_entity) = world.get_block_entity(&location).await {
            if let Some(inventory) = block_entity.get_inventory() {
                player
                    .open_handled_screen(&BarrelScreenFactory(inventory))
                    .await;
            }
        }
        BlockActionResult::Consume
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        let barrel_block_entity = BarrelBlockEntity::new(*pos);
        world.add_block_entity(Arc::new(barrel_block_entity)).await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: BlockStateId,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }
}
