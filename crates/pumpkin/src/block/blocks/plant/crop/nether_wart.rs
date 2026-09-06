use std::sync::Arc;

use pumpkin_data::{
    Block, BlockStateId,
    block_properties::NetherWartLikeProperties,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
        blocks::plant::{PlantBlockBase, crop::CropBlockBase},
    },
    world::World,
};

#[pumpkin_block("minecraft:nether_wart")]
pub struct NetherWartBlock;

impl BlockBehaviour for NetherWartBlock {
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

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        <Self as CropBlockBase>::random_tick(self, args.world, args.position);
    }
}

impl PlantBlockBase for NetherWartBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos);
        block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_NETHER_WART)
    }

    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down())
    }
}

impl CropBlockBase for NetherWartBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, pos)
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: BlockStateId, _block: &Block) -> i32 {
        let props = NetherWartLikeProperties::from_state_id(state);
        i32::from(props.age)
    }

    fn state_with_age(&self, block: &Block, state: BlockStateId, age: i32) -> BlockStateId {
        let mut props = NetherWartLikeProperties::from_state_id(state);
        props.age = age as u8;
        props.to_state_id(block)
    }

    fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos);
        let age = self.get_age(state, block);
        if age < self.max_age() && rand::rng().random_range(0..10) == 0 {
            world.set_block_state(
                pos,
                self.state_with_age(block, state, age + 1),
                BlockFlags::NOTIFY_NEIGHBORS,
            );
        }
    }
}
