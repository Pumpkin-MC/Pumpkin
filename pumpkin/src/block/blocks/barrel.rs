use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockState};
use pumpkin_data::item::Item;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::barrel::BarrelBlockEntity;
use tokio::sync::Mutex;

use crate::world::World;
use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
};

#[pumpkin_block("minecraft:barrel")]
pub struct BarrelBlock;

#[async_trait]
impl PumpkinBlock for BarrelBlock {
    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        world: &Arc<World>,
    ) {
        /*
        if let Some(barrel_block_entity) = world.get_block_entity(&location).await {
            player
                .open_handeled_screen(
                    Arc::new(Mutex::new(create_generic_9x3)),
                    TextComponent::new("Barrel"),
                )
                .await;
        }*/
    }

    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        _location: BlockPos,
        _item: &Item,
        server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
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
