use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, block_properties::GlowLichenLikeProperties,
};
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

            for _ in 0..self.search_range {
                let cur_pos = Self::search_position(pos, search_direction);
                let block_id = GenerationCache::get_block_state(chunk, &cur_pos.0).to_block_id();
                if !Self::is_air_or_water(chunk, cur_pos) && block_id != self.place_block {
                    break;
                }

                if self.place_growth_if_possible(chunk, cur_pos, random, &placement_directions) {
                    return true;
                }
            }
        }

        false
    }

    /// The position vanilla probes on iteration `i` of the search loop.
    ///
    /// `MultifaceGrowthFeature.place` keeps the cursor as a `MutableBlockPos` but re-seeds it
    /// from the *origin* on every iteration — the offset is taken from the origin position, not
    /// from the cursor it just wrote —
    /// so the search never walks away from `origin + direction`, whatever `search_range` says,
    /// and the loop just re-tests the same block. Advancing the cursor instead (what Pumpkin
    /// used to do) let glow lichen attach up to 20 blocks from where vanilla put it.
    fn search_position(origin: BlockPos, direction: BlockDirection) -> BlockPos {
        origin.offset(direction.to_offset())
    }

    /// Vanilla `MultifaceGrowthFeature.placeGrowthIfPossible` walks the direction list and stops
    /// at the first neighbour whose state is in `can_be_placed_on`. It asks the block for a
    /// placement state against that face, bails out if there is none, writes the growth, marks
    /// the position for post-processing, and then draws a `nextFloat` against
    /// `chance_of_spreading` — on success spreading from that face towards a random direction.
    ///
    /// The `nextFloat` is drawn on *every* successful placement (`chance_of_spreading` is 0.5
    /// for `glow_lichen`, 1.0 for `sculk_vein`), so leaving it out desynchronises the rest of
    /// the feature's stream, and the spread it gates places a second growth block.
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

            if !self.can_be_placed_on.contains(&neighbor_id) {
                continue;
            }

            // `getStateForPlacement` returning null aborts the whole call in vanilla; it does
            // not fall through to the next direction.
            let Some(state_id) = self.state_for_placement(chunk, pos, direction) else {
                return false;
            };
            chunk.set_block_state(&pos.0, BlockState::from_id(state_id));

            if random.next_f32() < self.chance_of_spreading {
                self.spread_from_face_toward_random_direction(
                    chunk, state_id, pos, direction, random,
                );
            }
            return true;
        }

        false
    }

    /// Vanilla `MultifaceBlock.getStateForPlacement`: `null` when the face cannot take the
    /// growth, otherwise the current growth state (or a fresh, possibly waterlogged one) with
    /// this face turned on.
    fn state_for_placement<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: BlockPos,
        direction: BlockDirection,
    ) -> Option<BlockStateId> {
        if !self.is_valid_state_for_placement(chunk, pos, direction) {
            return None;
        }

        let current = GenerationCache::get_block_state(chunk, &pos.0);
        let mut props = if current.to_block_id() == self.place_block {
            GlowLichenLikeProperties::from_state_id(current)
        } else {
            GlowLichenLikeProperties {
                down: false,
                up: false,
                north: false,
                south: false,
                west: false,
                east: false,
                waterlogged: current.to_block_id() == Block::WATER.id,
            }
        };
        Self::set_face(&mut props, direction, true);
        Some(props.to_state_id(Block::from_id(self.place_block)))
    }

    /// Vanilla `MultifaceBlock.isValidStateForPlacement`: every face is supported, a face the
    /// growth already has cannot be placed again, and the neighbour must offer a sturdy face
    /// towards this block (`MultifaceBlock.canAttachTo`).
    fn is_valid_state_for_placement<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let current = GenerationCache::get_block_state(chunk, &pos.0);
        if current.to_block_id() == self.place_block && self.has_face(current, direction) {
            return false;
        }
        let neighbor_pos = pos.offset(direction.to_offset());
        GenerationCache::get_block_state(chunk, &neighbor_pos.0)
            .to_state()
            .is_side_solid(direction.opposite())
    }

    /// Vanilla `MultifaceSpreader.spreadFromFaceTowardRandomDirection`: shuffle all six
    /// directions (five `nextInt` draws) and take the first that yields a spread position the
    /// growth can actually be placed into.
    fn spread_from_face_toward_random_direction<T: GenerationCache>(
        &self,
        chunk: &mut T,
        state: BlockStateId,
        pos: BlockPos,
        face: BlockDirection,
        random: &mut RandomGenerator,
    ) {
        for spread_direction in Self::shuffled_all_directions(random) {
            if let Some((spread_pos, spread_face)) = self.spread_pos_from_face_toward_direction(
                chunk,
                state,
                pos,
                face,
                spread_direction,
            ) && self.spread_to_face(chunk, spread_pos, spread_face)
            {
                return;
            }
        }
    }

    /// Vanilla `Direction.allShuffled` = `Util.shuffledCopy(Direction.values(), random)`, i.e.
    /// `[DOWN, UP, NORTH, SOUTH, WEST, EAST]` run through a downward Fisher-Yates pass that
    /// swaps each tail element with `nextInt(remaining)`, which always spends exactly five
    /// `nextInt` draws.
    fn shuffled_all_directions(random: &mut RandomGenerator) -> [BlockDirection; 6] {
        let mut directions = BlockDirection::all();
        for i in (1..directions.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            directions.swap(i, j);
        }
        directions
    }

    /// Vanilla `MultifaceSpreader.getSpreadFromFaceTowardDirection` with the default
    /// `SAME_POSITION, SAME_PLANE, WRAP_AROUND` order.
    fn spread_pos_from_face_toward_direction<T: GenerationCache>(
        &self,
        chunk: &T,
        state: BlockStateId,
        pos: BlockPos,
        face: BlockDirection,
        spread_direction: BlockDirection,
    ) -> Option<(BlockPos, BlockDirection)> {
        if spread_direction.to_axis() == face.to_axis() {
            return None;
        }
        if !self.has_face(state, face) || self.has_face(state, spread_direction) {
            return None;
        }

        Self::spread_candidates(pos, face, spread_direction)
            .into_iter()
            .find(|&(spread_pos, spread_face)| self.can_spread_into(chunk, spread_pos, spread_face))
    }

    /// `MultifaceSpreader.DEFAULT_SPREAD_ORDER`, in order: the same position facing the spread
    /// direction; the neighbour along the spread direction keeping the original face; then that
    /// neighbour offset again along the face, facing back against the spread direction.
    fn spread_candidates(
        pos: BlockPos,
        face: BlockDirection,
        spread_direction: BlockDirection,
    ) -> [(BlockPos, BlockDirection); 3] {
        [
            (pos, spread_direction),
            (pos.offset(spread_direction.to_offset()), face),
            (
                pos.offset(spread_direction.to_offset())
                    .offset(face.to_offset()),
                spread_direction.opposite(),
            ),
        ]
    }

    /// Vanilla `MultifaceSpreader.DefaultSpreaderConfig.canSpreadInto`: the target must be
    /// air, the growth itself or a water source, and the face must be placeable there.
    fn can_spread_into<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: BlockPos,
        face: BlockDirection,
    ) -> bool {
        let target = GenerationCache::get_block_state(chunk, &pos.0);
        let target_block = target.to_block_id();
        let replaceable = target.to_state().is_air()
            || target_block == self.place_block
            || target_block == Block::WATER.id;
        replaceable && self.is_valid_state_for_placement(chunk, pos, face)
    }

    /// Vanilla `MultifaceSpreader.spreadToFace`.
    fn spread_to_face<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        face: BlockDirection,
    ) -> bool {
        let Some(state_id) = self.state_for_placement(chunk, pos, face) else {
            return false;
        };
        chunk.set_block_state(&pos.0, BlockState::from_id(state_id));
        true
    }

    /// Vanilla `MultifaceBlock.hasFace` (`getValueOrElse(faceProperty, false)`).
    fn has_face(&self, state: BlockStateId, direction: BlockDirection) -> bool {
        if state.to_block_id() != self.place_block {
            return false;
        }
        let props = GlowLichenLikeProperties::from_state_id(state);
        match direction {
            BlockDirection::Down => props.down,
            BlockDirection::Up => props.up,
            BlockDirection::North => props.north,
            BlockDirection::South => props.south,
            BlockDirection::West => props.west,
            BlockDirection::East => props.east,
        }
    }

    const fn set_face(
        props: &mut GlowLichenLikeProperties,
        direction: BlockDirection,
        value: bool,
    ) {
        match direction {
            BlockDirection::Down => props.down = value,
            BlockDirection::Up => props.up = value,
            BlockDirection::North => props.north = value,
            BlockDirection::South => props.south = value,
            BlockDirection::West => props.west = value,
            BlockDirection::East => props.east = value,
        }
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
        let id = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();
        id == Block::AIR.id || id == Block::WATER.id
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::BlockDirection;
    use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

    use super::MultifaceGrowthFeature;

    /// `glow_lichen` has `search_range: 20`, but vanilla's loop re-offsets from the origin
    /// every iteration, so all 20 probes land on the same block, one step from the origin.
    #[test]
    fn search_never_walks_away_from_the_origin() {
        let origin = BlockPos(Vector3::new(7, 42, -3));
        for direction in [
            BlockDirection::Up,
            BlockDirection::Down,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::East,
            BlockDirection::West,
        ] {
            let offset = direction.to_offset();
            let expected = BlockPos(Vector3::new(
                origin.0.x + offset.x,
                origin.0.y + offset.y,
                origin.0.z + offset.z,
            ));
            for _ in 0..20 {
                assert_eq!(
                    MultifaceGrowthFeature::search_position(origin, direction),
                    expected,
                    "probe for {direction:?} moved away from origin + direction"
                );
            }
        }
    }

    /// Reference values from the real 26.2 server jar: four `Direction.allShuffled` calls in a
    /// row on a Xoroshiro stream seeded with 13579, plus the raw `nextInt()` after a single call
    /// — the sixth value of that stream, so a shuffle costs exactly five draws.
    #[test]
    fn shuffled_all_directions_matches_vanilla() {
        use pumpkin_util::random::{RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro};

        use BlockDirection::{Down, East, North, South, Up, West};
        let expected = [
            [West, Up, East, Down, South, North],
            [North, East, Up, South, Down, West],
            [Down, Up, North, East, South, West],
            [Up, Down, West, North, South, East],
        ];

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        for (call, expected) in expected.iter().enumerate() {
            assert_eq!(
                &MultifaceGrowthFeature::shuffled_all_directions(&mut random),
                expected,
                "allShuffled call #{call}"
            );
        }

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        MultifaceGrowthFeature::shuffled_all_directions(&mut random);
        assert_eq!(random.next_i32(), -1_266_923_548);
    }

    /// `MultifaceSpreader.DEFAULT_SPREAD_ORDER` for a growth on the `Up` face spreading north.
    #[test]
    fn spread_candidates_match_the_vanilla_spread_types() {
        let pos = BlockPos(Vector3::new(4, 70, 9));
        let candidates = MultifaceGrowthFeature::spread_candidates(
            pos,
            BlockDirection::Up,
            BlockDirection::North,
        );

        // SAME_POSITION: the same block, now carrying the spread direction as its face.
        assert_eq!(candidates[0], (pos, BlockDirection::North));
        // SAME_PLANE: one step along the spread direction, keeping the original face.
        assert_eq!(
            candidates[1],
            (BlockPos(Vector3::new(4, 70, 8)), BlockDirection::Up)
        );
        // WRAP_AROUND: around the corner, facing back the way it came.
        assert_eq!(
            candidates[2],
            (BlockPos(Vector3::new(4, 71, 8)), BlockDirection::South)
        );
    }
}
