use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, OnLandedUponArgs};

#[pumpkin_block("minecraft:slime_block")]
pub struct SlimeBlock;

impl BlockBehaviour for SlimeBlock {
    fn on_landed_upon<'a>(&'a self, args: OnLandedUponArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = args.entity.get_living_entity() {
                living.handle_fall_damage(args.entity, args.fall_distance, 0.0).await;
            }
        })
    }
}
