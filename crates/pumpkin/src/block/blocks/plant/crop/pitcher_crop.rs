use pumpkin_data::block_properties::{DoubleBlockHalf, PitcherCropLikeProperties};
use pumpkin_data::{Block, BlockStateId, tag, tag::Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    RandomTickArgs,
};

pub const MAX_AGE: u8 = 4;

#[pumpkin_block("minecraft:pitcher_crop")]
pub struct PitcherCropBlock;

impl PitcherCropBlock {
    #[must_use]
    pub fn can_survive(
        world: &dyn BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
        props: &PitcherCropLikeProperties,
    ) -> bool {
        let below = world.get_block(&pos.down());
        if props.half == DoubleBlockHalf::Lower {
            below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CROPS) || below == &Block::FARMLAND
        } else {
            below == &Block::PITCHER_CROP
        }
    }
}

impl BlockBehaviour for PitcherCropBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let props = PitcherCropLikeProperties::from_state_id(args.state.id);
        Self::can_survive(args.block_accessor, args.position, &props)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = PitcherCropLikeProperties::default(args.block);
        props.age = 0;
        props.half = DoubleBlockHalf::Lower;
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = PitcherCropLikeProperties::from_state_id(args.state_id);
        if !Self::can_survive(args.world, args.position, &props) {
            return Block::AIR.default_state.id;
        }

        if props.half == DoubleBlockHalf::Lower && props.age >= 3 {
            let above = args.world.get_block(&args.position.up());
            if above != &Block::PITCHER_CROP {
                return Block::AIR.default_state.id;
            }
        } else if props.half == DoubleBlockHalf::Upper {
            let below = args.world.get_block(&args.position.down());
            if below != &Block::PITCHER_CROP {
                return Block::AIR.default_state.id;
            }
        }

        args.state_id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = PitcherCropLikeProperties::from_state_id(state_id);
        if props.half == DoubleBlockHalf::Lower && props.age < MAX_AGE {
            let next_age = props.age + 1;
            if next_age >= 3 {
                let above_pos = args.position.up();
                let above_state = args.world.get_block_state(&above_pos);
                if !above_state.is_air()
                    && Block::from_state_id(above_state.id) != &Block::PITCHER_CROP
                {
                    return;
                }
            }
            props.age = next_age;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            if next_age >= 3 {
                let mut upper_props = props;
                upper_props.half = DoubleBlockHalf::Upper;
                args.world.set_block_state(
                    &args.position.up(),
                    upper_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let props = PitcherCropLikeProperties::from_state_id(args.state_id);
        if props.half == DoubleBlockHalf::Lower {
            props.age < MAX_AGE
        } else {
            let lower_state = args.world.get_block_state_id(&args.position.down());
            let lower_props = PitcherCropLikeProperties::from_state_id(lower_state);
            lower_props.age < MAX_AGE
        }
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let props = PitcherCropLikeProperties::from_state_id(args.state_id);
        let (lower_pos, lower_props) = if props.half == DoubleBlockHalf::Lower {
            (*args.position, props)
        } else {
            (
                args.position.down(),
                PitcherCropLikeProperties::from_state_id(
                    args.world.get_block_state_id(&args.position.down()),
                ),
            )
        };

        if lower_props.age < MAX_AGE {
            let next_age = lower_props.age + 1;
            let mut new_lower = lower_props;
            new_lower.age = next_age;
            args.world.set_block_state(
                &lower_pos,
                new_lower.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            if next_age >= 3 {
                let mut upper_props = new_lower;
                upper_props.half = DoubleBlockHalf::Upper;
                args.world.set_block_state(
                    &lower_pos.up(),
                    upper_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }
}
