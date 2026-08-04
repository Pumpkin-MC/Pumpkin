//! Generic face-attachment framework shared by blocks that can occupy any subset of
//! their six neighbouring faces (vanilla `MultifaceBlock`, `net/minecraft/world/level/block/MultifaceBlock.java`).
//!
//! No block in this codebase implements `MultifaceBlockBase` yet: this is pure,
//! inert infrastructure for future consumers (glow lichen, sculk vein).

use pumpkin_data::block_properties::{BlockProperties, GlowLichenLikeProperties};
use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

/// Bitmask of the six `BlockDirection`s a multiface block currently occupies.
///
/// Mirrors vanilla's `MultifaceBlock.pack`/`unpack`, which key a single byte off
/// `1 << direction.ordinal()`. `BlockDirection::to_index()` uses the same
/// Down/Up/North/South/West/East ordering as Java's `Direction.values()`, so this
/// bitmask is byte-identical to vanilla's (see `test_pack_matches_vanilla_ordinal`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FaceSet(u8);

impl FaceSet {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(0b0011_1111);

    #[must_use]
    const fn bit(direction: BlockDirection) -> u8 {
        1 << direction.to_index()
    }

    #[must_use]
    pub const fn contains(self, direction: BlockDirection) -> bool {
        self.0 & Self::bit(direction) != 0
    }

    #[must_use]
    pub const fn with(self, direction: BlockDirection) -> Self {
        Self(self.0 | Self::bit(direction))
    }

    #[must_use]
    pub const fn without(self, direction: BlockDirection) -> Self {
        Self(self.0 & !Self::bit(direction))
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn from_directions(directions: impl IntoIterator<Item = BlockDirection>) -> Self {
        directions.into_iter().fold(Self::EMPTY, Self::with)
    }

    pub fn iter(self) -> impl Iterator<Item = BlockDirection> {
        BlockDirection::all()
            .into_iter()
            .filter(move |&d| self.contains(d))
    }

    /// `MultifaceBlock.pack`.
    #[must_use]
    pub const fn pack(self) -> u8 {
        self.0
    }

    /// `MultifaceBlock.unpack`.
    #[must_use]
    pub const fn unpack(data: u8) -> Self {
        Self(data & Self::ALL.0)
    }
}

/// Bridges a real generated block-properties struct to the six face flags.
///
/// This is what `MultifaceBlockBase` logic operates through, so it never has to know
/// which concrete `BlockProperties` type a consumer block uses.
pub trait MultifaceProperties: BlockProperties {
    fn face(&self, direction: BlockDirection) -> bool;
    fn set_face(&mut self, direction: BlockDirection, value: bool);

    fn faces(&self) -> FaceSet {
        FaceSet::from_directions(BlockDirection::all().into_iter().filter(|&d| self.face(d)))
    }

    fn set_faces(&mut self, faces: FaceSet) {
        for direction in BlockDirection::all() {
            self.set_face(direction, faces.contains(direction));
        }
    }
}

impl MultifaceProperties for GlowLichenLikeProperties {
    fn face(&self, direction: BlockDirection) -> bool {
        match direction {
            BlockDirection::Down => self.r#down,
            BlockDirection::Up => self.r#up,
            BlockDirection::North => self.r#north,
            BlockDirection::South => self.r#south,
            BlockDirection::West => self.r#west,
            BlockDirection::East => self.r#east,
        }
    }

    fn set_face(&mut self, direction: BlockDirection, value: bool) {
        match direction {
            BlockDirection::Down => self.r#down = value,
            BlockDirection::Up => self.r#up = value,
            BlockDirection::North => self.r#north = value,
            BlockDirection::South => self.r#south = value,
            BlockDirection::West => self.r#west = value,
            BlockDirection::East => self.r#east = value,
        }
    }
}

/// `MultifaceBlock.canAttachTo`.
///
/// Vanilla checks `isFaceFull` against both the neighbour's block-support shape and
/// its collision shape. This codebase has no per-direction shape query yet, only the
/// coarser "is this side sturdy" flag (`BlockState::is_side_solid`, itself documented
/// as `isFaceSturdy()` in Java) already used for the same purpose by
/// `abstract_wall_mounting.rs`'s `WallMountedBlock::can_place_at`. Using it here is a
/// known simplification, not full shape-based parity, until per-direction voxel shapes
/// exist in `pumpkin_data`.
#[must_use]
pub const fn can_attach_to(
    neighbour_state: &BlockState,
    direction_towards_neighbour: BlockDirection,
) -> bool {
    neighbour_state.is_side_solid(direction_towards_neighbour.opposite())
}

/// `MultifaceBlock.canAttachTo(BlockGetter, BlockPos, Direction)` overload: looks the
/// neighbour up through `accessor` first.
#[must_use]
pub fn can_attach_to_pos(
    accessor: &dyn BlockAccessor,
    pos: &BlockPos,
    direction_towards_neighbour: BlockDirection,
) -> bool {
    let neighbour_pos = pos.offset(direction_towards_neighbour.to_offset());
    let neighbour_state = accessor.get_block_state(&neighbour_pos);
    can_attach_to(neighbour_state, direction_towards_neighbour)
}

/// `MultifaceBlock.hasAnyFace`.
#[must_use]
pub const fn has_any_face(faces: FaceSet) -> bool {
    !faces.is_empty()
}

/// `MultifaceBlock.hasAnyVacantFace`.
#[must_use]
pub const fn has_any_vacant_face(faces: FaceSet) -> bool {
    faces.0 != FaceSet::ALL.0
}

/// Shared face-attachment behaviour for a multiface block.
///
/// Every method here operates on a plain `FaceSet` rather than a concrete
/// block-properties struct, so a consumer converts to/from its own generated
/// properties type (see `MultifaceProperties`) around calls into this trait.
pub trait MultifaceBlockBase: Send + Sync {
    /// `MultifaceBlock.isFaceSupported`. Default: every face is supported.
    fn is_face_supported(&self, _direction: BlockDirection) -> bool {
        true
    }

    /// `MultifaceBlock.isValidStateForPlacement`.
    ///
    /// `existing_faces` is `Some` when the position being placed into already holds
    /// this same block (vanilla's `oldState.is(this)`), `None` otherwise.
    fn is_valid_state_for_placement(
        &self,
        accessor: &dyn BlockAccessor,
        existing_faces: Option<FaceSet>,
        placement_pos: &BlockPos,
        placement_direction: BlockDirection,
    ) -> bool {
        if !self.is_face_supported(placement_direction) {
            return false;
        }
        if existing_faces.is_some_and(|faces| faces.contains(placement_direction)) {
            return false;
        }
        can_attach_to_pos(accessor, placement_pos, placement_direction)
    }

    /// `MultifaceBlock.getStateForPlacement`, reduced to the face-set delta: returns
    /// the faces the placed block should end up with, or `None` if placement is invalid.
    fn faces_for_placement(
        &self,
        accessor: &dyn BlockAccessor,
        existing_faces: Option<FaceSet>,
        placement_pos: &BlockPos,
        placement_direction: BlockDirection,
    ) -> Option<FaceSet> {
        if !self.is_valid_state_for_placement(
            accessor,
            existing_faces,
            placement_pos,
            placement_direction,
        ) {
            return None;
        }
        Some(
            existing_faces
                .unwrap_or(FaceSet::EMPTY)
                .with(placement_direction),
        )
    }

    /// `MultifaceBlock.canSurvive`.
    fn can_survive(&self, accessor: &dyn BlockAccessor, pos: &BlockPos, faces: FaceSet) -> bool {
        has_any_face(faces) && faces.iter().all(|d| can_attach_to_pos(accessor, pos, d))
    }

    /// `MultifaceBlock.updateShape`, reduced to the face-set delta (the waterlogged
    /// re-scheduling side effect is the caller's responsibility, since it depends on
    /// the consumer's own block state, not on this framework).
    ///
    /// Returns `None` if the block should revert to air (no faces remain), otherwise
    /// the (possibly unchanged) face set.
    fn update_faces_for_neighbor(
        &self,
        faces: FaceSet,
        neighbour_state: &BlockState,
        direction_to_neighbour: BlockDirection,
    ) -> Option<FaceSet> {
        if faces.contains(direction_to_neighbour)
            && !can_attach_to(neighbour_state, direction_to_neighbour)
        {
            let updated = faces.without(direction_to_neighbour);
            has_any_face(updated).then_some(updated)
        } else {
            Some(faces)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;
    use pumpkin_world::world::BlockAccessor;
    use std::collections::HashMap;

    struct FakeAccessor {
        states: HashMap<BlockPos, &'static BlockState>,
        default: &'static BlockState,
    }

    impl FakeAccessor {
        fn new(default: &'static BlockState) -> Self {
            Self {
                states: HashMap::new(),
                default,
            }
        }

        fn with(mut self, pos: BlockPos, state: &'static BlockState) -> Self {
            self.states.insert(pos, state);
            self
        }
    }

    impl BlockAccessor for FakeAccessor {
        fn get_block(&self, position: &BlockPos) -> &'static pumpkin_data::Block {
            Block::from_state_id(self.get_block_state(position).id)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.states.get(position).copied().unwrap_or(self.default)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> pumpkin_data::BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static pumpkin_data::Block, &'static BlockState) {
            let state = self.get_block_state(position);
            (Block::from_state_id(state.id), state)
        }

        fn get_fluid(&self, _position: &BlockPos) -> pumpkin_data::fluid::Fluid {
            pumpkin_data::fluid::Fluid::EMPTY
        }
    }

    struct AllFacesSupported;
    impl MultifaceBlockBase for AllFacesSupported {}

    struct OnlyDownSupported;
    impl MultifaceBlockBase for OnlyDownSupported {
        fn is_face_supported(&self, direction: BlockDirection) -> bool {
            direction == BlockDirection::Down
        }
    }

    #[test]
    fn face_set_pack_matches_vanilla_ordinal() {
        // Java `Direction.values()` ordinal order: DOWN, UP, NORTH, SOUTH, WEST, EAST.
        // `BlockDirection::to_index()` must use the same order for the sculk charge
        // particle level-event byte (`numParticles << 6 | pack(faces)`) to render
        // correctly on vanilla clients.
        assert_eq!(BlockDirection::Down.to_index(), 0);
        assert_eq!(BlockDirection::Up.to_index(), 1);
        assert_eq!(BlockDirection::North.to_index(), 2);
        assert_eq!(BlockDirection::South.to_index(), 3);
        assert_eq!(BlockDirection::West.to_index(), 4);
        assert_eq!(BlockDirection::East.to_index(), 5);

        let faces = FaceSet::from_directions([BlockDirection::Up, BlockDirection::South]);
        assert_eq!(faces.pack(), (1 << 1) | (1 << 3));
        assert_eq!(FaceSet::unpack(faces.pack()), faces);
    }

    #[test]
    fn face_set_basic_ops() {
        let mut faces = FaceSet::EMPTY;
        assert!(faces.is_empty());
        assert!(!has_any_face(faces));

        faces = faces.with(BlockDirection::North);
        assert!(faces.contains(BlockDirection::North));
        assert!(!faces.contains(BlockDirection::South));
        assert!(has_any_face(faces));
        assert!(has_any_vacant_face(faces));

        faces = faces.without(BlockDirection::North);
        assert!(faces.is_empty());

        let full = FaceSet::from_directions(BlockDirection::all());
        assert_eq!(full, FaceSet::ALL);
        assert!(!has_any_vacant_face(full));
    }

    #[test]
    fn can_attach_to_reads_opposite_side_of_neighbour() {
        // A vein face pointing North attaches to a neighbour to the North whose
        // South-facing side (the side touching the vein) is sturdy.
        assert!(can_attach_to(
            Block::STONE.default_state,
            BlockDirection::North
        ));
        assert!(!can_attach_to(
            Block::AIR.default_state,
            BlockDirection::North
        ));
    }

    #[test]
    fn can_attach_to_pos_looks_up_neighbour_through_accessor() {
        let origin = BlockPos::new(0, 0, 0);
        let neighbour = origin.offset(BlockDirection::Up.to_offset());
        let accessor =
            FakeAccessor::new(Block::AIR.default_state).with(neighbour, Block::STONE.default_state);

        assert!(can_attach_to_pos(&accessor, &origin, BlockDirection::Up));
        assert!(!can_attach_to_pos(&accessor, &origin, BlockDirection::Down));
    }

    #[test]
    fn is_valid_state_for_placement_rejects_unsupported_face() {
        let block = OnlyDownSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::STONE.default_state);

        assert!(block.is_valid_state_for_placement(&accessor, None, &origin, BlockDirection::Down));
        assert!(!block.is_valid_state_for_placement(
            &accessor,
            None,
            &origin,
            BlockDirection::North
        ));
    }

    #[test]
    fn is_valid_state_for_placement_rejects_already_present_face() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::STONE.default_state);
        let existing = FaceSet::from_directions([BlockDirection::North]);

        assert!(!block.is_valid_state_for_placement(
            &accessor,
            Some(existing),
            &origin,
            BlockDirection::North
        ));
        assert!(block.is_valid_state_for_placement(
            &accessor,
            Some(existing),
            &origin,
            BlockDirection::South
        ));
    }

    #[test]
    fn is_valid_state_for_placement_rejects_non_sturdy_neighbour() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::AIR.default_state);

        assert!(!block.is_valid_state_for_placement(
            &accessor,
            None,
            &origin,
            BlockDirection::North
        ));
    }

    #[test]
    fn faces_for_placement_adds_the_new_face_to_existing_ones() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::STONE.default_state);
        let existing = FaceSet::from_directions([BlockDirection::North]);

        let result =
            block.faces_for_placement(&accessor, Some(existing), &origin, BlockDirection::South);
        assert_eq!(
            result,
            Some(FaceSet::from_directions([
                BlockDirection::North,
                BlockDirection::South
            ]))
        );
    }

    #[test]
    fn faces_for_placement_none_when_invalid() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::AIR.default_state);

        assert_eq!(
            block.faces_for_placement(&accessor, None, &origin, BlockDirection::North),
            None
        );
    }

    #[test]
    fn can_survive_false_with_no_faces() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::STONE.default_state);
        assert!(!block.can_survive(&accessor, &origin, FaceSet::EMPTY));
    }

    #[test]
    fn can_survive_true_when_every_present_face_attaches() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let accessor = FakeAccessor::new(Block::STONE.default_state);
        let faces = FaceSet::from_directions([BlockDirection::North, BlockDirection::Up]);
        assert!(block.can_survive(&accessor, &origin, faces));
    }

    #[test]
    fn can_survive_false_when_any_present_face_loses_its_neighbour() {
        let block = AllFacesSupported;
        let origin = BlockPos::new(0, 0, 0);
        let north = origin.offset(BlockDirection::North.to_offset());
        let accessor =
            FakeAccessor::new(Block::STONE.default_state).with(north, Block::AIR.default_state);
        let faces = FaceSet::from_directions([BlockDirection::North, BlockDirection::Up]);
        assert!(!block.can_survive(&accessor, &origin, faces));
    }

    #[test]
    fn update_faces_for_neighbor_keeps_face_when_still_attached() {
        let block = AllFacesSupported;
        let faces = FaceSet::from_directions([BlockDirection::North]);
        let result = block.update_faces_for_neighbor(
            faces,
            Block::STONE.default_state,
            BlockDirection::North,
        );
        assert_eq!(result, Some(faces));
    }

    #[test]
    fn update_faces_for_neighbor_drops_face_when_neighbour_no_longer_sturdy() {
        let block = AllFacesSupported;
        let faces = FaceSet::from_directions([BlockDirection::North, BlockDirection::Up]);
        let result =
            block.update_faces_for_neighbor(faces, Block::AIR.default_state, BlockDirection::North);
        assert_eq!(result, Some(FaceSet::from_directions([BlockDirection::Up])));
    }

    #[test]
    fn update_faces_for_neighbor_reverts_to_air_when_last_face_lost() {
        let block = AllFacesSupported;
        let faces = FaceSet::from_directions([BlockDirection::North]);
        let result =
            block.update_faces_for_neighbor(faces, Block::AIR.default_state, BlockDirection::North);
        assert_eq!(result, None);
    }

    #[test]
    fn update_faces_for_neighbor_ignores_unrelated_direction() {
        let block = AllFacesSupported;
        let faces = FaceSet::from_directions([BlockDirection::North]);
        let result =
            block.update_faces_for_neighbor(faces, Block::AIR.default_state, BlockDirection::South);
        assert_eq!(result, Some(faces));
    }

    #[test]
    fn glow_lichen_like_properties_round_trip_faces() {
        let mut props = GlowLichenLikeProperties::default(&Block::GLOW_LICHEN);
        let faces = FaceSet::from_directions([
            BlockDirection::Up,
            BlockDirection::North,
            BlockDirection::East,
        ]);
        props.set_faces(faces);
        assert_eq!(props.faces(), faces);
        assert!(props.r#up);
        assert!(props.r#north);
        assert!(props.r#east);
        assert!(!props.r#down);
        assert!(!props.r#south);
        assert!(!props.r#west);
    }
}
