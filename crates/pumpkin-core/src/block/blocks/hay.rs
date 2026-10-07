use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, OnLandedUponArgs};

#[pumpkin_block("minecraft:hay_block")]
pub struct HayBlock;

impl BlockBehaviour for HayBlock {
    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 0.2);
        }
    }
}
