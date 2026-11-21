use pumpkin_data::{
    Block, BlockState, BlockStateId, Rotation,
    block_properties::{Axis, PaleOakWoodLikeProperties},
};
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, OnLandedUponArgs, OnPlaceArgs};

#[pumpkin_block("minecraft:hay_block")]
pub struct HayBlock;

impl BlockBehaviour for HayBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = PaleOakWoodLikeProperties::default(args.block);
        props.axis = args.direction.to_axis();
        props.to_state_id(args.block)
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut props = PaleOakWoodLikeProperties::from_state_id(state_id);
        if matches!(
            rotation,
            Rotation::Clockwise90 | Rotation::CounterClockwise90
        ) {
            props.axis = match props.axis {
                Axis::X => Axis::Z,
                Axis::Z => Axis::X,
                Axis::Y => Axis::Y,
            };
        }
        BlockState::from_id(props.to_state_id(block))
    }

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 0.2);
        }
    }
}
