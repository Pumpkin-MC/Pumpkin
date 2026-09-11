use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{BambooLeaves, BambooLikeProperties},
    tag::Taggable,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

#[pumpkin_block("minecraft:bamboo_sapling")]
pub struct BambooSaplingBlock;

impl BlockBehaviour for BambooSaplingBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        let above = args.position.up();
        args.world.is_in_height_limit(above.0.y)
            && args.world.is_loaded(&above)
            && args.world.get_block_state(&above).is_air()
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        {
            grow_bamboo(args.world, args.position);
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            if args.block == &Block::BAMBOO_SAPLING
                && args.world.get_block(&args.position.up()) == &Block::BAMBOO
            {
                args.world.set_block_state(
                    args.position,
                    Block::BAMBOO.default_state.id,
                    BlockFlags::NOTIFY_NEIGHBORS,
                );
            }
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, args.world, args.position) {
            return Block::AIR.default_state.id;
        }
        if args.direction == BlockDirection::Up
            && args.world.get_block(args.neighbor_position) == &Block::BAMBOO
        {
            return Block::BAMBOO.default_state.id;
        }
        args.state_id
    }

    fn random_tick(&self, args: crate::block::RandomTickArgs<'_>) {
        let state_above = args.world.get_block_state(&args.position.up());
        if !state_above.is_air() || rand::rng().random_range(0..3) > 0 {
            return;
        }
        grow_bamboo(args.world, args.position);
    }
}

fn grow_bamboo(world: &std::sync::Arc<crate::world::World>, position: &BlockPos) {
    let mut props = BambooLikeProperties::from_state_id(Block::BAMBOO.default_state.id);
    props.leaves = BambooLeaves::Small;
    world.set_block_state(
        &position.up(),
        props.to_state_id(&Block::BAMBOO),
        BlockFlags::NOTIFY_ALL,
    );
}

impl PlantBlockBase for BambooSaplingBlock {
    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down())
    }

    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos);
        block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SUPPORTS_BAMBOO)
    }
}
