use pumpkin_data::block_properties::{Axis, DoubleBlockHalf, PitcherCropLikeProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, BlockDirection, BlockStateId, tag, tag::Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::block::blocks::plant::crop::{get_available_moisture, has_sufficient_light};
use crate::block::{
    BlockBehaviour, BonemealArgs, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnEntityCollisionArgs, OnPlaceArgs, RandomTickArgs,
};
use crate::world::World;

pub const MAX_AGE: u8 = 4;

#[pumpkin_block("minecraft:pitcher_crop")]
pub struct PitcherCropBlock;

impl PitcherCropBlock {
    #[must_use]
    pub fn can_survive(
        block_accessor: &dyn BlockAccessor,
        world: Option<&World>,
        pos: &BlockPos,
        props: &PitcherCropLikeProperties,
    ) -> bool {
        if props.half == DoubleBlockHalf::Lower {
            if let Some(world) = world
                && !has_sufficient_light(world, pos)
            {
                return false;
            }
            block_accessor
                .get_block(&pos.down())
                .has_tag(&tag::Block::MINECRAFT_SUPPORTS_CROPS)
        } else {
            let (below_block, below_state) = block_accessor.get_block_and_state(&pos.down());
            below_block == &Block::PITCHER_CROP
                && PitcherCropLikeProperties::from_state_id(below_state.id).half
                    == DoubleBlockHalf::Lower
        }
    }

    fn can_grow_into(world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(pos);
        state.is_air() || Block::from_state_id(state.id) == &Block::PITCHER_CROP
    }

    fn can_grow(
        world: &World,
        lower_pos: &BlockPos,
        lower_props: PitcherCropLikeProperties,
        new_age: u8,
    ) -> bool {
        let above = lower_pos.up();
        lower_props.age < MAX_AGE
            && has_sufficient_light(world, lower_pos)
            && world.is_in_build_limit(above)
            && (new_age < 3 || Self::can_grow_into(world, &above))
    }

    fn grow(
        world: &std::sync::Arc<World>,
        lower_pos: &BlockPos,
        lower_props: PitcherCropLikeProperties,
        increase: u8,
    ) {
        let updated_age = (lower_props.age + increase).min(MAX_AGE);
        if Self::can_grow(world, lower_pos, lower_props, updated_age) {
            let mut new_lower = lower_props;
            new_lower.age = updated_age;
            world.set_block_state(
                lower_pos,
                new_lower.to_state_id(&Block::PITCHER_CROP),
                BlockFlags::NOTIFY_LISTENERS,
            );
            if updated_age >= 3 {
                let mut upper_props = new_lower;
                upper_props.half = DoubleBlockHalf::Upper;
                world.set_block_state(
                    &lower_pos.up(),
                    upper_props.to_state_id(&Block::PITCHER_CROP),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn get_lower_half(
        world: &World,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) -> Option<(BlockPos, PitcherCropLikeProperties)> {
        if Block::from_state_id(state_id) == &Block::PITCHER_CROP {
            let props = PitcherCropLikeProperties::from_state_id(state_id);
            if props.half == DoubleBlockHalf::Lower {
                return Some((*pos, props));
            }
        }
        let lower_pos = pos.down();
        let (lower_block, lower_state_id) = world.get_block_and_state_id(&lower_pos);
        if lower_block == &Block::PITCHER_CROP {
            let lower_props = PitcherCropLikeProperties::from_state_id(lower_state_id);
            if lower_props.half == DoubleBlockHalf::Lower {
                return Some((lower_pos, lower_props));
            }
        }
        None
    }
}

impl BlockBehaviour for PitcherCropBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let props = PitcherCropLikeProperties::from_state_id(args.state.id);
        Self::can_survive(args.block_accessor, args.world, args.position, &props)
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
        if props.age >= 3
            && args.direction.to_axis() == Axis::Y
            && (props.half == DoubleBlockHalf::Lower) == (args.direction == BlockDirection::Up)
        {
            let neighbor_block = Block::from_state_id(args.neighbor_state_id);
            if neighbor_block != &Block::PITCHER_CROP
                || PitcherCropLikeProperties::from_state_id(args.neighbor_state_id).half
                    == props.half
            {
                return Block::AIR.default_state.id;
            }
        }
        if !Self::can_survive(args.world, Some(args.world), args.position, &props) {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let props = PitcherCropLikeProperties::from_state_id(args.state.id);
        if props.age < 3 {
            return;
        }
        let other_block_pos = match props.half {
            DoubleBlockHalf::Upper => args.position.down(),
            DoubleBlockHalf::Lower => args.position.up(),
        };
        let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_block_pos);
        if other_block == args.block {
            let other_props = PitcherCropLikeProperties::from_state_id(other_state_id);
            if other_props.half != props.half {
                args.world.break_block(
                    &other_block_pos,
                    None,
                    BlockFlags::SKIP_DROPS
                        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
                        | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if args.entity.get_entity().entity_type == &EntityType::RAVAGER
            && args.world.level_info.load().game_rules.mob_griefing
        {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let props = PitcherCropLikeProperties::from_state_id(state_id);
        if props.half == DoubleBlockHalf::Lower && props.age < MAX_AGE {
            let f = get_available_moisture(args.world, args.position, args.block);
            if rand::rng().random_range(0..=(25.0 / f).floor() as i32) == 0 {
                Self::grow(args.world, args.position, props, 1);
            }
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let Some((lower_pos, lower_props)) =
            Self::get_lower_half(args.world, args.position, args.state_id)
        else {
            return false;
        };
        Self::can_grow(args.world, &lower_pos, lower_props, lower_props.age + 1)
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        if let Some((lower_pos, lower_props)) =
            Self::get_lower_half(args.world, args.position, args.state_id)
        {
            Self::grow(args.world, &lower_pos, lower_props, 1);
        }
    }
}
