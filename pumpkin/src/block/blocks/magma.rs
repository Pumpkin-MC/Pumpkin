use std::sync::atomic::Ordering;

use pumpkin_data::damage::DamageType;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, OnEntityStepArgs};

#[pumpkin_block("minecraft:magma_block")]
pub struct MagmaBlock;

impl BlockBehaviour for MagmaBlock {
    fn on_entity_step<'a>(&'a self, args: OnEntityStepArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Only living entities take damage
            if args.entity.get_living_entity().is_none() {
                return;
            }

            let ent = args.entity.get_entity();

            // Don't damage if sneaking
            if ent.sneaking.load(Ordering::Relaxed) {
                return;
            }

            // Fire immune entities don't take damage
            if ent.fire_immune.load(Ordering::Relaxed) {
                return;
            }

            // Apply damage
            args.entity
                .damage(args.entity, 1.0, DamageType::HOT_FLOOR)
                .await;
        })
    }
}
