use crate::block::BlockIsReplacing;
use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, CanUpdateAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, PathComputationType,
};
use crate::entity::EntityBase;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomImpl, get_seed, legacy_rand::LegacyRand},
};
use pumpkin_world::world::{BlockAccessor, BlockFlags};

type SeaPickleProperties = pumpkin_data::block_properties::SeaPickleLikeProperties;

#[pumpkin_block("minecraft:sea_pickle")]
pub struct SeaPickleBlock;

impl SeaPickleBlock {
    const MAX_PICKLES: u8 = 4;
}

impl BlockBehaviour for SeaPickleBlock {
    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        SeaPickleProperties::from_state_id(args.state_id).waterlogged
            && args
                .world
                .get_block(&args.position.down())
                .has_tag(&tag::Block::MINECRAFT_CORAL_BLOCKS)
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let mut random = LegacyRand::from_seed(get_seed());
        let mut z_span = 1;
        let x_start = args.position.0.x - 2;
        let mut z_offset = 0;

        for x in 0..5 {
            for z in 0..z_span {
                let end_y = args.position.0.y + 1;
                for y in (end_y - 2)..end_y {
                    let position = BlockPos::new(x_start + x, y, args.position.0.z - z_offset + z);
                    if &position == args.position
                        || random.next_bounded_i32(6) != 0
                        || args.world.get_block(&position) != &Block::WATER
                        || !args
                            .world
                            .get_block(&position.down())
                            .has_tag(&tag::Block::MINECRAFT_CORAL_BLOCKS)
                    {
                        continue;
                    }

                    let mut properties = SeaPickleProperties::default(args.block);
                    properties.pickles =
                        (random.next_bounded_i32(i32::from(Self::MAX_PICKLES)) + 1) as u8;
                    args.world.set_block_state(
                        &position,
                        properties.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
            if x < 2 {
                z_span += 2;
                z_offset += 1;
            } else {
                z_span -= 2;
                z_offset -= 1;
            }
        }

        let mut properties = SeaPickleProperties::from_state_id(args.state_id);
        properties.pickles = Self::MAX_PICKLES;
        args.world.set_block_state(
            args.position,
            properties.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.player.get_entity().pose.load() != EntityPose::Crouching
            && let BlockIsReplacing::Itself(state_id) = args.replacing
        {
            let mut sea_pickle_prop = SeaPickleProperties::from_state_id(state_id);
            if sea_pickle_prop.pickles < Self::MAX_PICKLES {
                sea_pickle_prop.pickles += 1;
            }
            return sea_pickle_prop.to_state_id(args.block);
        }

        let mut sea_pickle_prop = SeaPickleProperties::default(args.block);
        sea_pickle_prop.waterlogged = args.replacing.water_source();
        sea_pickle_prop.to_state_id(args.block)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        args.player.get_entity().pose.load() != EntityPose::Crouching
            && SeaPickleProperties::from_state_id(args.state_id).pickles < Self::MAX_PICKLES
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

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl PlantBlockBase for SeaPickleBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        block_accessor
            .get_block_state(pos)
            .is_center_solid(BlockDirection::Up)
    }
}
