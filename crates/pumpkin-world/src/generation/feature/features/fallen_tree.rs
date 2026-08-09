use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::Axis};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    generation::{
        block_state_provider::BlockStateProvider,
        feature::features::{tree::TreeFeature, tree::decorator::TreeDecorator},
        proto_chunk::GenerationCache,
    },
    world::WorldPortalExt,
};

pub struct FallenTreeFeature {
    pub trunk_provider: BlockStateProvider,
    pub log_length: IntProvider,
    pub stump_decorators: Vec<TreeDecorator>,
    pub log_decorators: Vec<TreeDecorator>,
}

impl FallenTreeFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) -> bool {
        let stump = self.place_log_block(chunk, block_registry, random, origin, None);
        Self::decorate(
            chunk,
            block_registry,
            random,
            &[stump],
            &self.stump_decorators,
        );

        let direction = match random.next_bounded_i32(4) {
            0 => BlockDirection::North,
            1 => BlockDirection::East,
            2 => BlockDirection::South,
            _ => BlockDirection::West,
        };
        let log_length = self.log_length.get(random) - 2;
        let mut log_start =
            origin.offset_dir(direction.to_offset(), 2 + random.next_bounded_i32(2));
        Self::set_ground_height_for_log_start(chunk, &mut log_start);

        if Self::can_place_entire_log(chunk, log_length, &mut log_start, direction) {
            let mut logs = Vec::with_capacity(log_length.max(0) as usize);
            for _ in 0..log_length {
                logs.push(self.place_log_block(
                    chunk,
                    block_registry,
                    random,
                    log_start,
                    Some(direction),
                ));
                log_start = log_start.offset(direction.to_offset());
            }
            Self::decorate(chunk, block_registry, random, &logs, &self.log_decorators);
        }

        true
    }

    fn set_ground_height_for_log_start<T: GenerationCache>(chunk: &T, log_start: &mut BlockPos) {
        *log_start = log_start.up();
        for _ in 0..6 {
            if Self::may_place_on(chunk, *log_start) {
                return;
            }
            *log_start = log_start.down();
        }
    }

    fn can_place_entire_log<T: GenerationCache>(
        chunk: &T,
        log_length: i32,
        log_start: &mut BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let mut gap_in_ground = 0;
        for _ in 0..log_length {
            if !Self::valid_tree_pos(chunk, *log_start) {
                return false;
            }
            if Self::is_over_solid_ground(chunk, *log_start) {
                gap_in_ground = 0;
            } else {
                gap_in_ground += 1;
                if gap_in_ground > 2 {
                    return false;
                }
            }
            *log_start = log_start.offset(direction.to_offset());
        }
        *log_start = log_start.offset_dir(direction.to_offset(), -log_length);
        true
    }

    fn may_place_on<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        Self::valid_tree_pos(chunk, pos) && Self::is_over_solid_ground(chunk, pos)
    }

    fn is_over_solid_ground<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        GenerationCache::get_block_state(chunk, &pos.down().0)
            .to_state()
            .is_side_solid(BlockDirection::Up)
    }

    fn valid_tree_pos<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        TreeFeature::can_replace(state.to_state(), state.to_block_id())
    }

    fn place_log_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
        direction: Option<BlockDirection>,
    ) -> BlockPos {
        let state = self.trunk_provider.get(random, pos, chunk, block_registry);
        let state = direction.map_or(state, |direction| Self::sideways_state(state, direction));
        chunk.set_block_state(&pos.0, state);
        pos
    }

    fn sideways_state(
        state: &'static BlockState,
        direction: BlockDirection,
    ) -> &'static BlockState {
        let block = Block::from_state_id(state.id);
        let Some(original) = block.properties(state.id) else {
            return state;
        };
        let axis = match direction.to_axis() {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        };
        let mut properties = original.to_props();
        if let Some(index) = properties.iter().position(|(name, _)| *name == "axis") {
            properties[index] = ("axis", axis);
        }
        BlockState::from_id(block.from_properties(&properties).to_state_id(block))
    }

    fn decorate<T: GenerationCache>(
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
