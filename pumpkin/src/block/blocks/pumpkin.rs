use crate::block::pumpkin_block::UseWithItemArgs;
use crate::block::registry::BlockActionResult;
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use uuid::Uuid;
#[pumpkin_block("minecraft:pumpkin")]
pub struct PumpkinBlock;

#[async_trait]
impl crate::block::pumpkin_block::PumpkinBlock for PumpkinBlock {
    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.lock().await.item != &Item::SHEARS {
            return BlockActionResult::Continue;
        }
        // TODO: set direction
        args.world
            .set_block_state(
                args.location,
                Block::CARVED_PUMPKIN.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        let entity = Entity::new(
            Uuid::new_v4(),
            args.world.clone(),
            args.location.to_f64(),
            EntityType::ITEM,
            false,
        );
        let item_entity =
            Arc::new(ItemEntity::new(entity, ItemStack::new(4, &Item::PUMPKIN_SEEDS)).await);
        args.world.spawn_entity(item_entity).await;
        BlockActionResult::Consume
    }
}
