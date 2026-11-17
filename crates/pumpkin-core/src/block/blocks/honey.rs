use pumpkin_macros::pumpkin_block;

use crate::block::{
    BlockBehaviour, OnLandedUponArgs, PathComputationType, UpdateEntityMovementAfterFallOnArgs,
    stop_vertical_movement_after_fall,
};

#[pumpkin_block("minecraft:honey_block")]
pub struct HoneyBlock;

impl BlockBehaviour for HoneyBlock {
    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 0.2);
        }
    }

    fn update_entity_movement_after_fall_on(&self, args: UpdateEntityMovementAfterFallOnArgs<'_>) {
        stop_vertical_movement_after_fall(args.entity);
    }

    fn is_pathfindable(
        &self,
        _state: &pumpkin_data::BlockState,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }
}
