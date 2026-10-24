use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::GlowLichenLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct MultifaceGrowthFeature {
    pub place_block: pumpkin_data::BlockId,
    pub search_range: i32,
    pub can_place_on_floor: bool,
    pub can_place_on_ceiling: bool,
    pub can_place_on_wall: bool,
    pub chance_of_spreading: f32,
    pub can_be_placed_on: Vec<pumpkin_data::BlockId>,
}

impl MultifaceGrowthFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !Self::is_air_or_water(chunk, pos) {
            return false;
        }

        let shuffled_dirs = self.get_shuffled_directions(random);
        if self.place_growth_if_possible(chunk, pos, random, &shuffled_dirs) {
            return true;
        }

        for search_direction in shuffled_dirs {
            let placement_directions =
                self.get_shuffled_directions_except(random, search_direction.opposite());

            let cur_pos = pos.offset(search_direction.to_offset());
            let block_id = GenerationCache::get_block_state(chunk, &cur_pos.0).to_block_id();
            if !Self::is_air_or_water(chunk, cur_pos) && block_id != self.place_block {
                continue;
            }

            if self.place_growth_if_possible(chunk, cur_pos, random, &placement_directions) {
                return true;
            }
        }

        false
    }

    fn place_growth_if_possible<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        random: &mut RandomGenerator,
        placement_directions: &[BlockDirection],
    ) -> bool {
        for &direction in placement_directions {
            let neighbor_pos = pos.offset(direction.to_offset());
            let neighbor_id =
                GenerationCache::get_block_state(chunk, &neighbor_pos.0).to_block_id();

            if self.can_be_placed_on.contains(&neighbor_id) {
                self.place_face(chunk, pos, direction);

                if random.next_f32() < self.chance_of_spreading {
                    self.spread_from_face_toward_random_direction(chunk, pos, direction, random);
                }
                return true;
            }
        }

        false
    }

    fn spread_from_face_toward_random_direction<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        starting_face: BlockDirection,
        random: &mut RandomGenerator,
    ) {
        let mut dirs = [
            BlockDirection::Down,
            BlockDirection::Up,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ];
        for i in (1..dirs.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            dirs.swap(i, j);
        }

        for spread_direction in dirs {
            if spread_direction.to_axis() == starting_face.to_axis() {
                continue;
            }

            let cur_state = GenerationCache::get_block_state(chunk, &pos.0);
            if cur_state.to_block_id() == self.place_block {
                let cur_props = GlowLichenLikeProperties::from_state_id(cur_state);
                let has_spread_face = match spread_direction {
                    BlockDirection::Down => cur_props.down,
                    BlockDirection::Up => cur_props.up,
                    BlockDirection::North => cur_props.north,
                    BlockDirection::South => cur_props.south,
                    BlockDirection::West => cur_props.west,
                    BlockDirection::East => cur_props.east,
                };
                if has_spread_face {
                    continue;
                }
            }

            let candidates = [
                (pos, spread_direction),
                (pos.offset(spread_direction.to_offset()), starting_face),
                (
                    pos.offset(spread_direction.to_offset())
                        .offset(starting_face.to_offset()),
                    spread_direction.opposite(),
                ),
            ];

            for (target_pos, target_face) in candidates {
                if self.can_spread_into(chunk, target_pos, target_face) {
                    self.place_face(chunk, target_pos, target_face);
                    return;
                }
            }
        }
    }

    fn can_spread_into<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: BlockPos,
        face: BlockDirection,
    ) -> bool {
        let existing = GenerationCache::get_block_state(chunk, &pos.0);
        let id = existing.to_block_id();
        let replaceable =
            existing.to_state().is_air() || id == Block::WATER.id || id == self.place_block;
        if !replaceable {
            return false;
        }
        if id == self.place_block {
            let props = GlowLichenLikeProperties::from_state_id(existing);
            let has_face = match face {
                BlockDirection::Down => props.down,
                BlockDirection::Up => props.up,
                BlockDirection::North => props.north,
                BlockDirection::South => props.south,
                BlockDirection::West => props.west,
                BlockDirection::East => props.east,
            };
            if has_face {
                return false;
            }
        }
        let neighbour = pos.offset(face.to_offset());
        let neighbour_id = GenerationCache::get_block_state(chunk, &neighbour.0).to_block_id();
        self.can_be_placed_on.contains(&neighbour_id)
    }

    fn place_face<T: GenerationCache>(&self, chunk: &mut T, pos: BlockPos, face: BlockDirection) {
        let is_water =
            GenerationCache::get_block_state(chunk, &pos.0).to_block_id() == Block::WATER.id;
        let block = Block::from_id(self.place_block);

        let mut props = GlowLichenLikeProperties {
            down: face == BlockDirection::Down,
            up: face == BlockDirection::Up,
            north: face == BlockDirection::North,
            south: face == BlockDirection::South,
            west: face == BlockDirection::West,
            east: face == BlockDirection::East,
            waterlogged: is_water,
        };

        let cur_state = GenerationCache::get_block_state(chunk, &pos.0);
        if cur_state.to_block_id() == self.place_block
            && (self.place_block == Block::GLOW_LICHEN.id
                || self.place_block == Block::SCULK_VEIN.id)
        {
            let cur_props = GlowLichenLikeProperties::from_state_id(cur_state);
            props.down |= cur_props.down;
            props.up |= cur_props.up;
            props.north |= cur_props.north;
            props.south |= cur_props.south;
            props.west |= cur_props.west;
            props.east |= cur_props.east;
        }

        let state_id = props.to_state_id(block);
        chunk.set_block_state(&pos.0, BlockState::from_id(state_id));
    }

    fn get_valid_directions(&self) -> Vec<BlockDirection> {
        let mut dirs = Vec::with_capacity(6);
        if self.can_place_on_ceiling {
            dirs.push(BlockDirection::Up);
        }
        if self.can_place_on_floor {
            dirs.push(BlockDirection::Down);
        }
        if self.can_place_on_wall {
            dirs.extend(
                BlockDirection::horizontal_worldgen().map(BlockDirection::from_cardinal_direction),
            );
        }
        dirs
    }

    fn get_shuffled_directions(&self, random: &mut RandomGenerator) -> Vec<BlockDirection> {
        let mut dirs = self.get_valid_directions();
        for i in (1..dirs.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            dirs.swap(i, j);
        }
        dirs
    }

    fn get_shuffled_directions_except(
        &self,
        random: &mut RandomGenerator,
        exclude: BlockDirection,
    ) -> Vec<BlockDirection> {
        let mut dirs: Vec<BlockDirection> = self
            .get_valid_directions()
            .into_iter()
            .filter(|&d| d != exclude)
            .collect();
        for i in (1..dirs.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            dirs.swap(i, j);
        }
        dirs
    }

    fn is_air_or_water<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        state.to_state().is_air() || state.to_block_id() == Block::WATER.id
    }
}
