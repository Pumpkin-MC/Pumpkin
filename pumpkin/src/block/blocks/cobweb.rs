use crate::block::{BlockBehaviour, BlockFuture, OnEntityCollisionArgs};
use crate::entity::EntityBase;
use pumpkin_data::effect::StatusEffect;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;

#[pumpkin_block("minecraft:cobweb")]
pub struct CobwebBlock;

impl BlockBehaviour for CobwebBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = args.entity.get_entity();
            let vec = if let Some(living) = entity.get_living_entity()
                && living.has_effect(&StatusEffect::WEAVING).await
            {
                Vector3::new(0.5, 0.25, 0.5)
            } else {
                Vector3::new(0.25, 0.05, 0.25)
            };
            entity.slow_movement(args.state, vec).await;
        })
    }
}
