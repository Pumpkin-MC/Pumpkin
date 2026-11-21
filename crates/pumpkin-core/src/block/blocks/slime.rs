use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;

use crate::block::{BlockBehaviour, OnEntityStepArgs, OnLandedUponArgs};

#[pumpkin_block("minecraft:slime_block")]
pub struct SlimeBlock;

impl BlockBehaviour for SlimeBlock {
    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if !args.entity.get_entity().is_sneaking()
            && let Some(living) = args.entity.get_living_entity()
        {
            living.handle_fall_damage(args.entity, args.fall_distance, 0.0);
        }
    }

    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        let entity = args.entity.get_entity();
        let velocity = entity.velocity.load();
        let abs_y = velocity.y.abs();
        if abs_y < 0.1 && !args.entity.is_stepping_carefully() {
            let scale = 0.4 + abs_y * 0.2;
            entity.velocity.store(Vector3::new(
                velocity.x * scale,
                velocity.y,
                velocity.z * scale,
            ));
        }
    }
}
