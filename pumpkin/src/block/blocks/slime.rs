use std::sync::atomic::Ordering;

use pumpkin_macros::pumpkin_block;

use crate::block::{
    BlockBehaviour, BlockFuture, OnLandedUponArgs, UpdateEntityMovementAfterFallOnArgs,
};

#[pumpkin_block("minecraft:slime_block")]
pub struct SlimeBlock;

impl BlockBehaviour for SlimeBlock {
    fn on_landed_upon<'a>(&'a self, args: OnLandedUponArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = args.entity.get_living_entity() {
                living
                    .handle_fall_damage(args.entity, args.fall_distance, 0.0)
                    .await;
            }
        })
    }

    fn update_entity_movement_after_fall_on<'a>(
        &'a self,
        args: UpdateEntityMovementAfterFallOnArgs<'a>,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = args.entity.get_entity();
            let mut velocity = entity.velocity.load();

            if entity.sneaking.load(Ordering::Relaxed) {
                velocity.y = 0.0;
                entity.velocity.store(velocity);
                return;
            }

            if velocity.y < 0.0 {
                let factor = if args.entity.get_living_entity().is_some() {
                    1.0
                } else {
                    0.8
                };
                velocity.y = -velocity.y * factor;
                entity.velocity.store(velocity);
            }
        })
    }
}
