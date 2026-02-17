use pumpkin_macros::pumpkin_block;

use crate::block::{
    BlockBehaviour, BlockFuture, OnLandedUponArgs, UpdateEntityMovementAfterFallOnArgs,
    bounce_entity_after_fall,
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
        Box::pin(async move { bounce_entity_after_fall(args.entity, 1.0) })
    }
}
