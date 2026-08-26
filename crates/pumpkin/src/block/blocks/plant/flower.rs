use pumpkin_data::BlockStateId;
use pumpkin_macros::pumpkin_block_from_tag;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs};

#[pumpkin_block_from_tag("minecraft:small_flowers")]
pub struct FlowerBlock;

impl BlockBehaviour for FlowerBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }
}

impl PlantBlockBase for FlowerBlock {}
