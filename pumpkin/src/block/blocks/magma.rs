use std::sync::atomic::Ordering;

use pumpkin_data::damage::DamageType;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, OnEntityCollisionArgs};

#[pumpkin_block("minecraft:magma_block")]
pub struct MagmaBlock;

impl BlockBehaviour for MagmaBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Only damage players standing on the top face (supporting block)
            if args.entity.get_player().is_none() {
                return;
            }

            // Check if the entity is standing on this block (UP face/top)
            let ent = args.entity.get_entity();
            if let Some(supporting) = ent.supporting_block_pos.load() {
                if supporting == *args.position {
                    // If the entity is sneaking/crouching, do not damage
                    if ent.sneaking.load(Ordering::Relaxed) {
                        return;
                    }
                } else {
                    // Not standing on top of this block (don't apply magma damage)
                    return;
                }
            } else {
                return;
            }

            // Damage the entity
            let _ = args
                .entity
                .damage(args.entity, 1.0, DamageType::HOT_FLOOR)
                .await;
        })
    }
}
