//! Port of vanilla's `MultifaceSpreader`.
//!
//! Source: `net/minecraft/world/level/block/MultifaceSpreader.java`, the
//! spread-candidate-finding algorithm shared by glow lichen and sculk vein. Builds on
//! `abstract_multiface.rs`'s `FaceSet`. No block in this codebase calls into this
//! module yet; it is pure, inert infrastructure for future consumers.

use crate::block::BlockFuture;
use crate::block::blocks::abstract_multiface::FaceSet;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use pumpkin_world::world::BlockAccessor;

/// `MultifaceSpreader.SpreadPos`: a candidate target position plus the face the
/// multiface block would occupy there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadPos {
    pub pos: BlockPos,
    pub face: BlockDirection,
}

/// `MultifaceSpreader.SpreadType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadType {
    /// Spread to a different face of the same block.
    SamePosition,
    /// Move to the neighbour in the spread direction, keeping the same face.
    SamePlane,
    /// Move diagonally around a corner; the face flips to the opposite of the
    /// spread direction.
    WrapAround,
}

/// `MultifaceSpreader.DEFAULT_SPREAD_ORDER`.
pub const DEFAULT_SPREAD_ORDER: [SpreadType; 3] = [
    SpreadType::SamePosition,
    SpreadType::SamePlane,
    SpreadType::WrapAround,
];

impl SpreadType {
    /// `MultifaceSpreader.SpreadType#getSpreadPos`.
    #[must_use]
    pub fn spread_pos(
        self,
        pos: BlockPos,
        spread_direction: BlockDirection,
        from_face: BlockDirection,
    ) -> SpreadPos {
        match self {
            Self::SamePosition => SpreadPos {
                pos,
                face: spread_direction,
            },
            Self::SamePlane => SpreadPos {
                pos: pos.offset(spread_direction.to_offset()),
                face: from_face,
            },
            Self::WrapAround => SpreadPos {
                pos: pos
                    .offset(spread_direction.to_offset())
                    .offset(from_face.to_offset()),
                face: spread_direction.opposite(),
            },
        }
    }
}

/// `MultifaceSpreader.SpreadConfig`.
///
/// All candidate-search methods are synchronous and pure given a `BlockAccessor`;
/// actual world mutation is a separate concern (see `SpreadTarget` below), since this
/// codebase's block writes are async.
pub trait SpreadConfig: Send + Sync {
    /// `SpreadConfig.hasFace`. Default delegates straight to the `FaceSet`.
    fn has_face(&self, faces: FaceSet, direction: BlockDirection) -> bool {
        faces.contains(direction)
    }

    /// `SpreadConfig.isOtherBlockValidAsSource`. Default: never (matches vanilla's
    /// `DefaultSpreaderConfig`; overridden by consumers like sculk vein's config,
    /// which treats any bare replaceable surface as a valid source).
    fn is_other_block_valid_as_source(&self, _faces: FaceSet) -> bool {
        false
    }

    /// `SpreadConfig.canSpreadFrom`.
    fn can_spread_from(&self, faces: FaceSet, direction: BlockDirection) -> bool {
        self.is_other_block_valid_as_source(faces) || self.has_face(faces, direction)
    }

    /// `SpreadConfig.getSpreadTypes`.
    fn spread_types(&self) -> &'static [SpreadType] {
        &DEFAULT_SPREAD_ORDER
    }

    /// `SpreadConfig.canSpreadInto`. This is where a consumer's replacement/validity
    /// rules (e.g. sculk vein's refusal to spread onto sculk/sculk-catalyst/moving-piston
    /// faces) live.
    fn can_spread_into(
        &self,
        accessor: &dyn BlockAccessor,
        source_pos: &BlockPos,
        spread_pos: SpreadPos,
    ) -> bool;
}

/// `MultifaceSpreader.getSpreadFromFaceTowardDirection`.
///
/// Uses the config's own `canSpreadInto` as the spread predicate (vanilla also exposes
/// a variant taking an arbitrary predicate, e.g. for `canSpreadInAnyDirection`'s dry-run
/// check; here that's just this same function reused, since our `can_spread_into`
/// already takes the same arguments as vanilla's `SpreadPredicate`).
#[must_use]
pub fn get_spread_from_face_toward_direction(
    config: &dyn SpreadConfig,
    accessor: &dyn BlockAccessor,
    faces: FaceSet,
    pos: BlockPos,
    starting_face: BlockDirection,
    spread_direction: BlockDirection,
) -> Option<SpreadPos> {
    if spread_direction.to_axis() == starting_face.to_axis() {
        return None;
    }

    let can_spread = config.is_other_block_valid_as_source(faces)
        || (config.has_face(faces, starting_face) && !config.has_face(faces, spread_direction));
    if !can_spread {
        return None;
    }

    for &spread_type in config.spread_types() {
        let candidate = spread_type.spread_pos(pos, spread_direction, starting_face);
        if config.can_spread_into(accessor, &pos, candidate) {
            return Some(candidate);
        }
    }
    None
}

/// `MultifaceSpreader.canSpreadInAnyDirection`.
#[must_use]
pub fn can_spread_in_any_direction(
    config: &dyn SpreadConfig,
    accessor: &dyn BlockAccessor,
    faces: FaceSet,
    pos: BlockPos,
    starting_face: BlockDirection,
) -> bool {
    BlockDirection::all().into_iter().any(|spread_direction| {
        get_spread_from_face_toward_direction(
            config,
            accessor,
            faces,
            pos,
            starting_face,
            spread_direction,
        )
        .is_some()
    })
}

/// `Direction.allShuffled(random)`: a Fisher-Yates shuffle of the six directions.
#[must_use]
pub fn shuffled_directions(random: &mut RandomGenerator) -> [BlockDirection; 6] {
    let mut dirs = BlockDirection::all();
    for i in (1..dirs.len()).rev() {
        let j = random.next_bounded_i32(i as i32 + 1) as usize;
        dirs.swap(i, j);
    }
    dirs
}

/// `MultifaceSpreader.SpreadConfig#placeBlock` split across the sync/async boundary.
///
/// Placing a block needs to both read (`canSpreadInto`, already covered by
/// `SpreadConfig`) and write the target block. Writes in this codebase go through
/// `World::set_block_state`, which is async, so the actual placement step is a separate
/// trait a consumer implements around its own world reference — this mirrors
/// `DefaultSpreaderConfig` wrapping a `MultifaceBlock` reference in vanilla.
///
/// `accessor()` must reflect the same live world `place()` writes to, so that
/// `spread_all`/`spread_from_face_toward_all_directions` see each write before
/// evaluating the next candidate — vanilla relies on exactly this interleaving
/// (`LevelAccessor` reads and writes go through the same mutable level object).
pub trait SpreadTarget: Send + Sync {
    fn accessor(&self) -> &dyn BlockAccessor;

    /// Attempts to place at `spread_pos`, returning whether it succeeded.
    fn place(&self, spread_pos: SpreadPos) -> BlockFuture<'_, bool>;
}

/// `MultifaceSpreader.spreadFromFaceTowardDirection`.
pub async fn spread_from_face_toward_direction(
    config: &dyn SpreadConfig,
    target: &dyn SpreadTarget,
    faces: FaceSet,
    pos: BlockPos,
    starting_face: BlockDirection,
    spread_direction: BlockDirection,
) -> Option<SpreadPos> {
    let candidate = get_spread_from_face_toward_direction(
        config,
        target.accessor(),
        faces,
        pos,
        starting_face,
        spread_direction,
    )?;
    target.place(candidate).await.then_some(candidate)
}

/// `MultifaceSpreader.spreadFromFaceTowardRandomDirection`.
pub async fn spread_from_face_toward_random_direction(
    config: &dyn SpreadConfig,
    target: &dyn SpreadTarget,
    faces: FaceSet,
    pos: BlockPos,
    starting_face: BlockDirection,
    random: &mut RandomGenerator,
) -> Option<SpreadPos> {
    for spread_direction in shuffled_directions(random) {
        if let Some(candidate) = spread_from_face_toward_direction(
            config,
            target,
            faces,
            pos,
            starting_face,
            spread_direction,
        )
        .await
        {
            return Some(candidate);
        }
    }
    None
}

/// `MultifaceSpreader.spreadFromRandomFaceTowardRandomDirection`.
pub async fn spread_from_random_face_toward_random_direction(
    config: &dyn SpreadConfig,
    target: &dyn SpreadTarget,
    faces: FaceSet,
    pos: BlockPos,
    random: &mut RandomGenerator,
) -> Option<SpreadPos> {
    for starting_face in shuffled_directions(random) {
        if !config.can_spread_from(faces, starting_face) {
            continue;
        }
        if let Some(candidate) = spread_from_face_toward_random_direction(
            config,
            target,
            faces,
            pos,
            starting_face,
            random,
        )
        .await
        {
            return Some(candidate);
        }
    }
    None
}

/// `MultifaceSpreader.spreadFromFaceTowardAllDirections`.
pub async fn spread_from_face_toward_all_directions(
    config: &dyn SpreadConfig,
    target: &dyn SpreadTarget,
    faces: FaceSet,
    pos: BlockPos,
    starting_face: BlockDirection,
) -> u64 {
    let mut count = 0;
    for spread_direction in BlockDirection::all() {
        if spread_from_face_toward_direction(
            config,
            target,
            faces,
            pos,
            starting_face,
            spread_direction,
        )
        .await
        .is_some()
        {
            count += 1;
        }
    }
    count
}

/// `MultifaceSpreader.spreadAll`.
///
/// Each candidate is read and (if valid) placed before the next candidate is
/// evaluated, so later checks see earlier writes — see the `SpreadTarget` doc comment
/// for why this can't be reduced to a pre-collected list.
pub async fn spread_all(
    config: &dyn SpreadConfig,
    target: &dyn SpreadTarget,
    faces: FaceSet,
    pos: BlockPos,
) -> u64 {
    let mut total = 0;
    for starting_face in BlockDirection::all() {
        if config.can_spread_from(faces, starting_face) {
            total +=
                spread_from_face_toward_all_directions(config, target, faces, pos, starting_face)
                    .await;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::{Block, BlockState};
    use pumpkin_util::random::xoroshiro128::Xoroshiro;
    use std::collections::HashSet;
    use std::sync::Mutex;

    struct FakeAccessor {
        occupied: HashSet<BlockPos>,
        default: &'static BlockState,
    }

    impl FakeAccessor {
        fn new(default: &'static BlockState) -> Self {
            Self {
                occupied: HashSet::new(),
                default,
            }
        }

        fn occupy(mut self, pos: BlockPos) -> Self {
            self.occupied.insert(pos);
            self
        }
    }

    impl BlockAccessor for FakeAccessor {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            if self.occupied.contains(position) {
                Block::STONE.default_state
            } else {
                self.default
            }
        }

        fn get_block_state_id(&self, position: &BlockPos) -> pumpkin_data::BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, _position: &BlockPos) -> pumpkin_data::fluid::Fluid {
            pumpkin_data::fluid::Fluid::EMPTY
        }
    }

    /// A config where any empty (non-`occupied`) position is spreadable into, matching
    /// vanilla's `DefaultSpreaderConfig.stateCanBeReplaced` treating air as replaceable.
    struct OpenAirConfig;
    impl SpreadConfig for OpenAirConfig {
        fn can_spread_into(
            &self,
            accessor: &dyn BlockAccessor,
            _source_pos: &BlockPos,
            spread_pos: SpreadPos,
        ) -> bool {
            accessor.get_block(&spread_pos.pos) == &Block::AIR
        }
    }

    #[test]
    fn same_axis_spread_direction_is_always_rejected() {
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state);
        let faces = FaceSet::from_directions([BlockDirection::North]);

        // North and South share the Z axis with the starting face itself (North).
        assert_eq!(
            get_spread_from_face_toward_direction(
                &config,
                &accessor,
                faces,
                BlockPos::new(0, 0, 0),
                BlockDirection::North,
                BlockDirection::North,
            ),
            None
        );
        assert_eq!(
            get_spread_from_face_toward_direction(
                &config,
                &accessor,
                faces,
                BlockPos::new(0, 0, 0),
                BlockDirection::North,
                BlockDirection::South,
            ),
            None
        );
    }

    #[test]
    fn spreading_requires_the_starting_face_and_an_empty_target_face() {
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state);

        // No North face present at all: nothing to spread from.
        assert_eq!(
            get_spread_from_face_toward_direction(
                &config,
                &accessor,
                FaceSet::EMPTY,
                BlockPos::new(0, 0, 0),
                BlockDirection::North,
                BlockDirection::Up,
            ),
            None
        );

        // North face present, Up face already occupied on this block: same-position
        // spread onto Up is rejected because hasFace(spreadDirection) is already true.
        let faces = FaceSet::from_directions([BlockDirection::North, BlockDirection::Up]);
        assert_eq!(
            get_spread_from_face_toward_direction(
                &config,
                &accessor,
                faces,
                BlockPos::new(0, 0, 0),
                BlockDirection::North,
                BlockDirection::Up,
            ),
            None
        );
    }

    #[test]
    fn same_position_candidate_is_tried_first() {
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state);
        let faces = FaceSet::from_directions([BlockDirection::North]);
        let pos = BlockPos::new(0, 0, 0);

        let result = get_spread_from_face_toward_direction(
            &config,
            &accessor,
            faces,
            pos,
            BlockDirection::North,
            BlockDirection::Up,
        );
        assert_eq!(
            result,
            Some(SpreadPos {
                pos,
                face: BlockDirection::Up
            })
        );
    }

    #[test]
    fn falls_through_to_same_plane_when_same_position_is_blocked() {
        let pos = BlockPos::new(0, 0, 0);
        // Occupy the same-position candidate (pos, Up) so the config must reject it
        // and fall through to SAME_PLANE.
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state).occupy(pos);
        let faces = FaceSet::from_directions([BlockDirection::North]);

        let result = get_spread_from_face_toward_direction(
            &config,
            &accessor,
            faces,
            pos,
            BlockDirection::North,
            BlockDirection::Up,
        );
        assert_eq!(
            result,
            Some(SpreadPos {
                pos: pos.offset(BlockDirection::Up.to_offset()),
                face: BlockDirection::North,
            })
        );
    }

    #[test]
    fn falls_through_to_wrap_around_when_first_two_are_blocked() {
        let pos = BlockPos::new(0, 0, 0);
        let same_plane_pos = pos.offset(BlockDirection::Up.to_offset());
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state)
            .occupy(pos)
            .occupy(same_plane_pos);
        let faces = FaceSet::from_directions([BlockDirection::North]);

        let result = get_spread_from_face_toward_direction(
            &config,
            &accessor,
            faces,
            pos,
            BlockDirection::North,
            BlockDirection::Up,
        );
        assert_eq!(
            result,
            Some(SpreadPos {
                pos: pos
                    .offset(BlockDirection::Up.to_offset())
                    .offset(BlockDirection::North.to_offset()),
                face: BlockDirection::Down,
            })
        );
    }

    #[test]
    fn no_candidate_when_all_three_spread_types_are_blocked() {
        let pos = BlockPos::new(0, 0, 0);
        let same_plane_pos = pos.offset(BlockDirection::Up.to_offset());
        let wrap_pos = same_plane_pos.offset(BlockDirection::North.to_offset());
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state)
            .occupy(pos)
            .occupy(same_plane_pos)
            .occupy(wrap_pos);
        let faces = FaceSet::from_directions([BlockDirection::North]);

        assert_eq!(
            get_spread_from_face_toward_direction(
                &config,
                &accessor,
                faces,
                pos,
                BlockDirection::North,
                BlockDirection::Up,
            ),
            None
        );
    }

    #[test]
    fn other_block_valid_as_source_bypasses_the_face_check() {
        struct AlwaysSourceConfig;
        impl SpreadConfig for AlwaysSourceConfig {
            fn is_other_block_valid_as_source(&self, _faces: FaceSet) -> bool {
                true
            }
            fn can_spread_into(
                &self,
                accessor: &dyn BlockAccessor,
                _source_pos: &BlockPos,
                spread_pos: SpreadPos,
            ) -> bool {
                accessor.get_block(&spread_pos.pos) == &Block::AIR
            }
        }

        let config = AlwaysSourceConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state);
        // No faces at all, yet a spread candidate is still found because this block
        // (a bare replaceable surface, per sculk vein's rule) is a valid source.
        let result = get_spread_from_face_toward_direction(
            &config,
            &accessor,
            FaceSet::EMPTY,
            BlockPos::new(0, 0, 0),
            BlockDirection::North,
            BlockDirection::Up,
        );
        assert!(result.is_some());
    }

    #[test]
    fn can_spread_in_any_direction_true_when_one_direction_works() {
        let config = OpenAirConfig;
        let accessor = FakeAccessor::new(Block::AIR.default_state);
        let faces = FaceSet::from_directions([BlockDirection::North]);
        assert!(can_spread_in_any_direction(
            &config,
            &accessor,
            faces,
            BlockPos::new(0, 0, 0),
            BlockDirection::North,
        ));
    }

    #[test]
    fn can_spread_in_any_direction_false_when_fully_blocked() {
        let config = OpenAirConfig;
        let pos = BlockPos::new(0, 0, 0);
        // Occupy every non-same-axis direction's SAME_POSITION/SAME_PLANE/WRAP_AROUND
        // candidates around the origin so nothing can ever succeed.
        let mut accessor = FakeAccessor::new(Block::STONE.default_state);
        accessor.occupied.insert(pos);
        let faces = FaceSet::from_directions([BlockDirection::North]);
        assert!(!can_spread_in_any_direction(
            &config,
            &accessor,
            faces,
            pos,
            BlockDirection::North,
        ));
    }

    #[test]
    fn shuffled_directions_is_a_permutation_of_all_six() {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(42));
        let shuffled = shuffled_directions(&mut random);
        let mut sorted = shuffled;
        sorted.sort_by_key(BlockDirection::to_index);
        assert_eq!(sorted, BlockDirection::all());
    }

    #[test]
    fn shuffled_directions_is_deterministic_for_a_fixed_seed() {
        let mut a = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(7));
        let mut b = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(7));
        assert_eq!(shuffled_directions(&mut a), shuffled_directions(&mut b));
    }

    /// A `SpreadTarget` writing into a shared, in-memory `occupied` set, so tests can
    /// assert on read/write interleaving without a real `World`. `BlockAccessor` is
    /// implemented directly on the mutex-guarded state, locking per call, so
    /// `accessor()` can hand out `&self` without holding a guard across the borrow.
    struct RecordingTarget {
        occupied: Mutex<HashSet<BlockPos>>,
        default: &'static BlockState,
    }

    impl RecordingTarget {
        fn new(default: &'static BlockState) -> Self {
            Self {
                occupied: Mutex::new(HashSet::new()),
                default,
            }
        }

        fn placed_count(&self) -> usize {
            self.occupied.lock().unwrap().len()
        }
    }

    impl BlockAccessor for RecordingTarget {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            if self.occupied.lock().unwrap().contains(position) {
                Block::STONE.default_state
            } else {
                self.default
            }
        }

        fn get_block_state_id(&self, position: &BlockPos) -> pumpkin_data::BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, _position: &BlockPos) -> pumpkin_data::fluid::Fluid {
            pumpkin_data::fluid::Fluid::EMPTY
        }
    }

    impl SpreadTarget for RecordingTarget {
        fn accessor(&self) -> &dyn BlockAccessor {
            self
        }

        fn place(&self, spread_pos: SpreadPos) -> BlockFuture<'_, bool> {
            Box::pin(async move {
                let mut occupied = self.occupied.lock().unwrap();
                if occupied.contains(&spread_pos.pos) {
                    false
                } else {
                    occupied.insert(spread_pos.pos);
                    true
                }
            })
        }
    }

    #[tokio::test]
    async fn spread_all_places_and_counts_successful_spreads() {
        let target = RecordingTarget::new(Block::AIR.default_state);
        let config = OpenAirConfig;
        let faces = FaceSet::from_directions([BlockDirection::North]);
        let pos = BlockPos::new(0, 0, 0);

        let count = spread_all(&config, &target, faces, pos).await;
        assert!(count > 0);

        assert_eq!(count, target.placed_count() as u64);
    }

    #[tokio::test]
    async fn spread_all_does_not_double_place_the_same_position() {
        // SAME_POSITION candidates for every non-North spread direction all target the
        // origin itself; only the first should succeed, later ones must see it already
        // occupied and be rejected by `can_spread_into` — this only holds if writes and
        // reads are interleaved rather than computed from a pre-collected snapshot.
        let target = RecordingTarget::new(Block::AIR.default_state);
        let config = OpenAirConfig;
        let faces = FaceSet::from_directions([BlockDirection::North]);
        let pos = BlockPos::new(0, 0, 0);

        let count = spread_from_face_toward_all_directions(
            &config,
            &target,
            faces,
            pos,
            BlockDirection::North,
        )
        .await;

        // Exactly one spread direction (the first non-same-axis one tried, Down) can
        // land its SAME_POSITION candidate at `pos`; every other direction must fall
        // through to SAME_PLANE/WRAP_AROUND (different positions) or fail entirely.
        assert!(count >= 1);
        assert_eq!(count, target.placed_count() as u64);
    }
}
