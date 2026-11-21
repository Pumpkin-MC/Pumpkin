use pumpkin_data::{
    Block, BlockDirection, BlockId,
    block_properties::{
        BlockProperties, DoubleBlockHalf, EastWall, NorthWall, PaleMossCarpetLikeProperties,
        SmallDripleafLikeProperties, SouthWall, TallSeagrassLikeProperties, WestWall,
    },
    fluid::Fluid,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::block_state_provider::BlockStateProvider,
    world::{BlockAccessor, WorldPortalExt},
};

pub struct SimpleBlockFeature {
    pub to_place: BlockStateProvider,
    pub schedule_tick: Option<bool>,
}

impl SimpleBlockFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let Some(state) = self
            .to_place
            .get_optional(block_registry, chunk, random, pos)
        else {
            return false;
        };
        let block = Block::from_state_id(state.id);
        let block_accessor: &dyn BlockAccessor = chunk;
        if !block_registry.can_place_at(block, state, block_accessor, &pos) {
            return false;
        }

        if block.id == BlockId::SMALL_DRIPLEAF
            || TallSeagrassLikeProperties::handles_block_id(block.id)
        {
            let above_pos = pos.up();
            let above_raw = GenerationCache::get_block_state(chunk, &above_pos.0);
            let above_state = above_raw.to_state();
            if !above_state.is_air() {
                let state_is_water = block.id == BlockId::TALL_SEAGRASS
                    || state.is_waterlogged()
                    || matches!(
                        Fluid::from_state_id(state.id),
                        Some(f) if *f == Fluid::WATER || *f == Fluid::FLOWING_WATER
                    );
                let above_is_water = Self::is_water_at(chunk, &above_pos);
                if state_is_water != above_is_water || !above_state.replaceable() {
                    return false;
                }
            }
            if block.id == BlockId::SMALL_DRIPLEAF {
                let mut lower_props = SmallDripleafLikeProperties::from_state_id(state.id);
                lower_props.half = DoubleBlockHalf::Lower;
                lower_props.waterlogged = Self::is_water_at(chunk, &pos);
                chunk.set_block_state(&pos.0, lower_props.to_state_id(block).to_state());

                let mut upper_props = lower_props;
                upper_props.half = DoubleBlockHalf::Upper;
                upper_props.waterlogged = Self::is_water_at(chunk, &above_pos);
                chunk.set_block_state(&above_pos.0, upper_props.to_state_id(block).to_state());
            } else {
                let mut lower_props = TallSeagrassLikeProperties::from_state_id(state.id);
                lower_props.half = DoubleBlockHalf::Lower;
                chunk.set_block_state(&pos.0, lower_props.to_state_id(block).to_state());

                let mut upper_props = lower_props;
                upper_props.half = DoubleBlockHalf::Upper;
                chunk.set_block_state(&above_pos.0, upper_props.to_state_id(block).to_state());
            }
        } else if block.id == BlockId::PALE_MOSS_CARPET {
            Self::place_pale_moss_carpet(chunk, pos, random);
        } else {
            chunk.set_block_state(&pos.0, state);
        }
        true
    }

    fn is_water_at<T: GenerationCache>(chunk: &T, pos: &BlockPos) -> bool {
        let id = GenerationCache::get_block_state(chunk, &pos.0);
        id.is_waterlogged()
            || matches!(
                Fluid::from_state_id(id),
                Some(f) if *f == Fluid::WATER || *f == Fluid::FLOWING_WATER
            )
    }

    fn can_support_carpet_face<T: GenerationCache>(
        chunk: &T,
        pos: BlockPos,
        dir: BlockDirection,
    ) -> bool {
        let neighbor_pos = pos.offset(dir.to_offset());
        let neighbor_raw = GenerationCache::get_block_state(chunk, &neighbor_pos.0);
        neighbor_raw.to_state().is_side_solid(dir.opposite())
    }

    fn updated_pale_moss_carpet_state<T: GenerationCache>(
        mut props: PaleMossCarpetLikeProperties,
        chunk: &T,
        pos: BlockPos,
        mut create_sides: bool,
    ) -> PaleMossCarpetLikeProperties {
        create_sides |= props.bottom;
        let above_raw = GenerationCache::get_block_state(chunk, &pos.up().0);
        let below_raw = GenerationCache::get_block_state(chunk, &pos.down().0);
        let above_props = (above_raw.to_block_id() == BlockId::PALE_MOSS_CARPET)
            .then(|| PaleMossCarpetLikeProperties::from_state_id(above_raw));
        let below_props = (below_raw.to_block_id() == BlockId::PALE_MOSS_CARPET)
            .then(|| PaleMossCarpetLikeProperties::from_state_id(below_raw));

        if Self::can_support_carpet_face(chunk, pos, BlockDirection::North) {
            if create_sides {
                props.north = NorthWall::Low;
            }
            if props.north == NorthWall::Low {
                if above_props.is_some_and(|ap| ap.north != NorthWall::None && !ap.bottom) {
                    props.north = NorthWall::Tall;
                }
                if !props.bottom && below_props.is_some_and(|bp| bp.north == NorthWall::None) {
                    props.north = NorthWall::None;
                }
            }
        } else {
            props.north = NorthWall::None;
        }

        if Self::can_support_carpet_face(chunk, pos, BlockDirection::East) {
            if create_sides {
                props.east = EastWall::Low;
            }
            if props.east == EastWall::Low {
                if above_props.is_some_and(|ap| ap.east != EastWall::None && !ap.bottom) {
                    props.east = EastWall::Tall;
                }
                if !props.bottom && below_props.is_some_and(|bp| bp.east == EastWall::None) {
                    props.east = EastWall::None;
                }
            }
        } else {
            props.east = EastWall::None;
        }

        if Self::can_support_carpet_face(chunk, pos, BlockDirection::South) {
            if create_sides {
                props.south = SouthWall::Low;
            }
            if props.south == SouthWall::Low {
                if above_props.is_some_and(|ap| ap.south != SouthWall::None && !ap.bottom) {
                    props.south = SouthWall::Tall;
                }
                if !props.bottom && below_props.is_some_and(|bp| bp.south == SouthWall::None) {
                    props.south = SouthWall::None;
                }
            }
        } else {
            props.south = SouthWall::None;
        }

        if Self::can_support_carpet_face(chunk, pos, BlockDirection::West) {
            if create_sides {
                props.west = WestWall::Low;
            }
            if props.west == WestWall::Low {
                if above_props.is_some_and(|ap| ap.west != WestWall::None && !ap.bottom) {
                    props.west = WestWall::Tall;
                }
                if !props.bottom && below_props.is_some_and(|bp| bp.west == WestWall::None) {
                    props.west = WestWall::None;
                }
            }
        } else {
            props.west = WestWall::None;
        }

        props
    }

    fn place_pale_moss_carpet<T: GenerationCache>(
        chunk: &mut T,
        pos: BlockPos,
        random: &mut RandomGenerator,
    ) {
        let base_props = PaleMossCarpetLikeProperties::default(&Block::PALE_MOSS_CARPET);
        let adjusted = Self::updated_pale_moss_carpet_state(base_props, chunk, pos, true);
        chunk.set_block_state(
            &pos.0,
            adjusted.to_state_id(&Block::PALE_MOSS_CARPET).to_state(),
        );

        let above = pos.up();
        let above_prev = GenerationCache::get_block_state(chunk, &above.0);
        let is_carpet_above = above_prev.to_block_id() == BlockId::PALE_MOSS_CARPET;
        let can_place_topper = (!is_carpet_above
            || !PaleMossCarpetLikeProperties::from_state_id(above_prev).bottom)
            && (is_carpet_above || above_prev.to_state().replaceable());

        if can_place_topper {
            let mut no_base = PaleMossCarpetLikeProperties::default(&Block::PALE_MOSS_CARPET);
            no_base.bottom = false;
            let mut topper = Self::updated_pale_moss_carpet_state(no_base, chunk, above, true);
            // Direction.Plane.HORIZONTAL order: NORTH, EAST, SOUTH, WEST
            if topper.north != NorthWall::None && !random.next_bool() {
                topper.north = NorthWall::None;
            }
            if topper.east != EastWall::None && !random.next_bool() {
                topper.east = EastWall::None;
            }
            if topper.south != SouthWall::None && !random.next_bool() {
                topper.south = SouthWall::None;
            }
            if topper.west != WestWall::None && !random.next_bool() {
                topper.west = WestWall::None;
            }
            let has_faces = topper.bottom
                || topper.north != NorthWall::None
                || topper.east != EastWall::None
                || topper.south != SouthWall::None
                || topper.west != WestWall::None;
            let topper_id = topper.to_state_id(&Block::PALE_MOSS_CARPET);
            if has_faces && topper_id != above_prev {
                chunk.set_block_state(&above.0, topper_id.to_state());
                let updated_bottom =
                    Self::updated_pale_moss_carpet_state(adjusted, chunk, pos, true);
                chunk.set_block_state(
                    &pos.0,
                    updated_bottom
                        .to_state_id(&Block::PALE_MOSS_CARPET)
                        .to_state(),
                );
            }
        }
    }
}
