use pumpkin_data::Block;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::NetherWartLikeProperties;
use pumpkin_macros::pumpkin_block;
use rand::RngExt;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::{BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs};

type BeetrootProperties = NetherWartLikeProperties;

#[pumpkin_block("minecraft:beetroots")]
pub struct BeetrootBlock;

impl BlockBehaviour for BeetrootBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        <Self as CropBlockBase>::is_valid_bonemeal_target(self, args.world, args.position)
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        <Self as CropBlockBase>::perform_bonemeal(self, args.world, args.position);
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as CropBlockBase>::can_plant_on_top(self, args.block_accessor, &args.position.down())
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

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::rng().random_range(0..3) == 0 {
            <Self as CropBlockBase>::random_tick(self, args.world, args.position);
        }
    }
}

impl PlantBlockBase for BeetrootBlock {}

impl CropBlockBase for BeetrootBlock {
    fn bonemeal_age_increase(&self) -> i32 {
        rand::rng().random_range(2..=5) / 3
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: BlockStateId, _block: &Block) -> i32 {
        let props = BeetrootProperties::from_state_id(state);
        i32::from(props.age)
    }

    fn state_with_age(&self, block: &Block, state: BlockStateId, age: i32) -> BlockStateId {
        let mut props = BeetrootProperties::from_state_id(state);
        props.age = age as u8;
        props.to_state_id(block)
    }
}
