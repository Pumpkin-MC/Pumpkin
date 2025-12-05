use crate::block::registry::BlockActionResult;
use crate::block::{BlockFuture, UseWithItemArgs};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use uuid::Uuid;

#[pumpkin_block("minecraft:pumpkin")]
pub struct PumpkinBlock;

impl crate::block::BlockBehaviour for PumpkinBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if args.item_stack.lock().await.item != &Item::SHEARS {
                return BlockActionResult::Pass;
            }
            let mut props = WallTorchLikeProperties::default(&Block::CARVED_PUMPKIN);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(&Block::CARVED_PUMPKIN),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            let entity = Entity::new(
                Uuid::new_v4(),
                args.world.clone(),
                args.position.to_f64(),
                &EntityType::ITEM,
                false,
            );
            let item_entity =
                Arc::new(ItemEntity::new(entity, ItemStack::new(4, &Item::PUMPKIN_SEEDS)).await);
            args.world.spawn_entity(item_entity).await;
            BlockActionResult::Consume
        })
    }
}
