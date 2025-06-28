use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::server::Server;
use crate::world::World;
use crate::{block::registry::BlockActionResult, entity::player::Player};
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use uuid::Uuid;
#[pumpkin_block("minecraft:pumpkin")]
pub struct PumpkinBlock;

#[async_trait]
impl crate::block::pumpkin_block::PumpkinBlock for PumpkinBlock {
    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        pos: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        if item != &Item::SHEARS {
            return BlockActionResult::Continue;
        }
        // TODO: set direction
        world
            .set_block_state(
                &pos,
                Block::CARVED_PUMPKIN.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            pos.to_f64(),
            EntityType::ITEM,
            false,
        );
        let item_entity =
            Arc::new(ItemEntity::new(entity, ItemStack::new(4, &Item::PUMPKIN_SEEDS)).await);
        world.spawn_entity(item_entity).await;
        BlockActionResult::Consume
    }
}
