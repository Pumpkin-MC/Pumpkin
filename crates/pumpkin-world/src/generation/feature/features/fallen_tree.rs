use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use super::tree::{TreeFeature, decorator::TreeDecorator};
use crate::{
    generation::{block_state_provider::BlockStateProvider, proto_chunk::GenerationCache},
    world::WorldPortalExt,
};

pub struct FallenTreeFeature {
    pub trunk_provider: BlockStateProvider,
    pub log_length: IntProvider,
    pub stump_decorators: Vec<TreeDecorator>,
    pub log_decorators: Vec<TreeDecorator>,
}

impl FallenTreeFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        self.place_stump(chunk, block_registry, random, pos);
        let direction =
            BlockDirection::from_cardinal_direction(BlockDirection::random_horizontal(random));
        let log_length = self.log_length.get(random) - 2;
        let mut log_start_pos =
            pos.offset(direction.to_offset() * (2 + random.next_bounded_i32(2)));
        Self::set_ground_height_for_fallen_log_start_pos(chunk, &mut log_start_pos);
        if Self::can_place_entire_fallen_log(chunk, log_length, log_start_pos, direction) {
            self.place_fallen_log(
                chunk,
                block_registry,
                random,
                log_length,
                log_start_pos,
                direction,
            );
        }
        true
    }

    fn set_ground_height_for_fallen_log_start_pos<T: GenerationCache>(
        chunk: &T,
        log_start_pos: &mut BlockPos,
    ) {
        *log_start_pos = log_start_pos.up();
        for _ in 0..6 {
            if Self::may_place_on(chunk, *log_start_pos) {
                return;
            }
            *log_start_pos = log_start_pos.down();
        }
    }

    fn place_stump<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        stump_pos: BlockPos,
    ) {
        let stump = self.place_log_block(chunk, block_registry, random, stump_pos, None);
        Self::decorate_logs(
            chunk,
            block_registry,
            random,
            &[stump],
            &self.stump_decorators,
        );
    }

    fn can_place_entire_fallen_log<T: GenerationCache>(
        chunk: &T,
        log_length: i32,
        mut log_start_pos: BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let mut gap_in_ground = 0;
        let offset = direction.to_offset();

        for _ in 0..log_length {
            let state = GenerationCache::get_block_state(chunk, &log_start_pos.0);
            if !TreeFeature::can_replace(state.to_state(), state.to_block_id()) {
                return false;
            }

            if Self::is_over_solid_ground(chunk, log_start_pos) {
                gap_in_ground = 0;
            } else {
                gap_in_ground += 1;
                if gap_in_ground > 2 {
                    return false;
                }
            }

            log_start_pos = log_start_pos.offset(offset);
        }

        true
    }

    fn place_fallen_log<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        log_length: i32,
        mut log_start_pos: BlockPos,
        direction: BlockDirection,
    ) {
        let mut fallen_log = Vec::with_capacity(log_length.max(0) as usize);
        let offset = direction.to_offset();

        for _ in 0..log_length {
            let placed = self.place_log_block(
                chunk,
                block_registry,
                random,
                log_start_pos,
                Some(direction),
            );
            fallen_log.push(placed);
            log_start_pos = log_start_pos.offset(offset);
        }

        Self::decorate_logs(
            chunk,
            block_registry,
            random,
            &fallen_log,
            &self.log_decorators,
        );
    }

    fn may_place_on<T: GenerationCache>(chunk: &T, block_pos: BlockPos) -> bool {
        let state = GenerationCache::get_block_state(chunk, &block_pos.0);
        TreeFeature::can_replace(state.to_state(), state.to_block_id())
            && Self::is_over_solid_ground(chunk, block_pos)
    }

    fn is_over_solid_ground<T: GenerationCache>(chunk: &T, block_pos: BlockPos) -> bool {
        let below = block_pos.down();
        GenerationCache::get_block_state(chunk, &below.0)
            .to_state()
            .is_side_solid(BlockDirection::Up)
    }

    fn place_log_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        block_pos: BlockPos,
        direction: Option<BlockDirection>,
    ) -> BlockPos {
        let mut state = self
            .trunk_provider
            .get(random, block_pos, chunk, block_registry);
        if let Some(dir) = direction {
            let block = Block::from_state_id(state.id);
            let axis_str = match dir {
                BlockDirection::East | BlockDirection::West => "x",
                BlockDirection::North | BlockDirection::South => "z",
                _ => "y",
            };
            if let Some(props) = block.properties(state.id) {
                let prop_list = props.to_props();
                if prop_list.iter().any(|(k, _)| *k == "axis") {
                    let new_props: Vec<(&str, &str)> = prop_list
                        .iter()
                        .map(|(k, v)| (*k, if *k == "axis" { axis_str } else { *v }))
                        .collect();
                    let new_props_obj = block.from_properties(new_props.as_slice());
                    let new_state_id = new_props_obj.to_state_id(block);
                    state = BlockState::from_id(new_state_id);
                }
            }
        }
        chunk.set_block_state(&block_pos.0, state);
        block_pos
    }

    fn decorate_logs<T: GenerationCache>(
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        logs: &[BlockPos],
        decorators: &[TreeDecorator],
    ) {
        for decorator in decorators {
            decorator.generate(chunk, block_registry, random, &[], logs, &[]);
        }
    }
}
