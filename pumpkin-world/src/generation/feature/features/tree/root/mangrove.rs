use pumpkin_data::{BlockDirection, BlockState, block_properties::HorizontalFacing, fluid::Fluid};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::TreeFeature;
use crate::generation::proto_chunk::GenerationCache;

pub struct AboveRootPlacement {
    pub above_root_provider: BlockStateProvider,
    pub above_root_placement_chance: f32,
}

pub struct MangroveRootPlacement {
    pub can_grow_through: &'static [u16],
    pub muddy_roots_in: &'static [u16],
    pub muddy_roots_provider: BlockStateProvider,
    pub max_root_width: i32,
    pub max_root_length: i32,
    pub random_skew_chance: f32,
}

pub struct MangroveRootPlacer {
    pub trunk_offset_y: IntProvider,
    pub root_provider: BlockStateProvider,
    pub above_root_placement: Option<AboveRootPlacement>,
    pub mangrove_root_placement: MangroveRootPlacement,
}

impl MangroveRootPlacer {
    pub fn trunk_offset(&self, pos: BlockPos, random: &mut RandomGenerator) -> BlockPos {
        pos.up_height(self.trunk_offset_y.get(random))
    }

    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        trunk_pos: BlockPos,
    ) -> Option<Vec<BlockPos>> {
        let mut roots: Vec<BlockPos> = Vec::new();

        let mut walker = pos;
        while walker.0.y < trunk_pos.0.y {
            if !self.can_grow_through(chunk, walker) {
                return None;
            }
            walker = walker.up();
        }

        roots.push(trunk_pos.down());

        for dir in BlockDirection::horizontal() {
            let start = trunk_pos.offset_dir(dir.to_offset(), 1);
            let mut offshoots: Vec<BlockPos> = Vec::new();
            if !self.can_grow(chunk, random, start, dir, trunk_pos, &mut offshoots, 0) {
                return None;
            }
            roots.extend(offshoots);
            roots.push(start);
        }

        let mut placed: Vec<BlockPos> = Vec::new();
        for root_pos in roots {
            if self.place_roots(chunk, random, root_pos) {
                placed.push(root_pos);
            }
        }
        Some(placed)
    }

    #[expect(clippy::too_many_arguments)]
    fn can_grow<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        direction: HorizontalFacing,
        origin: BlockPos,
        offshoots: &mut Vec<BlockPos>,
        root_length: i32,
    ) -> bool {
        let max = self.mangrove_root_placement.max_root_length;
        if root_length == max || offshoots.len() > max as usize {
            return false;
        }
        for next in self.get_offshoot_positions(pos, direction, random, origin) {
            if self.can_grow_through(chunk, next) {
                offshoots.push(next);
                if !self.can_grow(
                    chunk,
                    random,
                    next,
                    direction,
                    origin,
                    offshoots,
                    root_length + 1,
                ) {
                    return false;
                }
            }
        }
        true
    }

    fn get_offshoot_positions(
        &self,
        pos: BlockPos,
        direction: HorizontalFacing,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) -> Vec<BlockPos> {
        let down = pos.down();
        let sideways = pos.offset_dir(direction.to_offset(), 1);
        let dist = pos.manhattan_distance(origin);
        let max_width = self.mangrove_root_placement.max_root_width;
        let skew = self.mangrove_root_placement.random_skew_chance;

        if dist > max_width - 3 && dist <= max_width {
            if random.next_f32() < skew {
                vec![down, sideways.down()]
            } else {
                vec![down]
            }
        } else if dist > max_width || random.next_f32() < skew {
            vec![down]
        } else if random.next_bool() {
            vec![sideways]
        } else {
            vec![down]
        }
    }

    fn can_grow_through<T: GenerationCache>(&self, chunk: &T, pos: BlockPos) -> bool {
        let raw = GenerationCache::get_block_state(chunk, &pos.0);
        let block_id = raw.to_block_id();
        TreeFeature::can_replace(raw.to_state(), block_id)
            || self
                .mangrove_root_placement
                .can_grow_through
                .contains(&block_id)
    }

    fn place_roots<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let existing_id = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();
        if self
            .mangrove_root_placement
            .muddy_roots_in
            .contains(&existing_id)
        {
            let state = self
                .mangrove_root_placement
                .muddy_roots_provider
                .get(random, pos);
            let state = Self::apply_waterlogging(chunk, pos, state);
            chunk.set_block_state(&pos.0, state);
            return true;
        }
        if !self.can_grow_through(chunk, pos) {
            return false;
        }
        let state = self.root_provider.get(random, pos);
        let state = Self::apply_waterlogging(chunk, pos, state);
        chunk.set_block_state(&pos.0, state);
        if let Some(above) = &self.above_root_placement {
            let above_pos = pos.up();
            if random.next_f32() < above.above_root_placement_chance && chunk.is_air(&above_pos.0) {
                let above_state = above.above_root_provider.get(random, above_pos);
                let above_state = Self::apply_waterlogging(chunk, above_pos, above_state);
                chunk.set_block_state(&above_pos.0, above_state);
            }
        }
        true
    }

    fn apply_waterlogging<T: GenerationCache>(
        chunk: &T,
        pos: BlockPos,
        state: &'static BlockState,
    ) -> &'static BlockState {
        if state.is_waterlogged() {
            return state;
        }
        let (fluid, _) = GenerationCache::get_fluid_and_fluid_state(chunk, &pos.0);
        if fluid != Fluid::WATER {
            return state;
        }
        state.with_waterlogged().unwrap_or(state)
    }
}
