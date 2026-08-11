//! Vanilla-faithful sculk-vein spreading logic.
//!
//! Reference: `net.minecraft.world.level.block.SculkVeinBlock` and
//! `net.minecraft.world.level.block.MultifaceSpreader` (mc-26_2).

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::GlowLichenLikeProperties;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::RandomImpl;

use super::SculkLevel;
use super::is_sculk_replaceable;

/// Multiface spread positions — same three types as vanilla.
#[derive(Debug, Clone, Copy)]
pub enum SpreadType {
    /// Place at the same position, facing `spread_direction`.
    SamePosition,
    /// Place at the neighbour in `spread_direction`, facing `from_face`.
    SamePlane,
    /// Wrap around: place at neighbour + `from_face`, facing opposite.
    WrapAround,
}

impl SpreadType {
    /// Computes the target position and the face the new vein should have.
    pub fn spread_pos(
        self,
        pos: BlockPos,
        spread_direction: BlockDirection,
        from_face: BlockDirection,
    ) -> (BlockPos, BlockDirection) {
        match self {
            Self::SamePosition => (pos, spread_direction),
            Self::SamePlane => (pos.offset(spread_direction.to_offset()), from_face),
            Self::WrapAround => (
                pos.offset(spread_direction.to_offset())
                    .offset(from_face.to_offset()),
                spread_direction.opposite(),
            ),
        }
    }
}

/// Vanilla spread order.
pub const DEFAULT_SPREAD_ORDER: [SpreadType; 3] = [
    SpreadType::SamePosition,
    SpreadType::SamePlane,
    SpreadType::WrapAround,
];

/// Rules for sculk-vein spreading. Mirrors `SculkVeinBlock` + `MultifaceSpreader`.
pub struct VeinRules;

impl VeinRules {
    /// Attempts to place sculk at a support block adjacent to `pos`.
    /// Mirrors vanilla `SculkVeinBlock.attemptPlaceSculk`.
    pub fn attempt_place_sculk(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        random: &mut RandomGenerator,
        replaceable: impl Fn(BlockId) -> bool,
    ) -> bool {
        let Some(state) = level.sculk_get(pos) else {
            return false;
        };

        // Vanilla shuffles the support directions with the level random
        // source (`Direction.allShuffled(random)`).
        let mut support_order = BlockDirection::all();
        for i in (1..support_order.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            support_order.swap(i, j);
        }

        for support in support_order {
            if !Self::has_face(state, support) {
                continue;
            }
            let support_pos = pos.offset(support.to_offset());
            let Some(support_state) = level.sculk_get(support_pos) else {
                continue;
            };
            let support_id = support_state.to_block_id();
            if !replaceable(support_id) {
                continue;
            }
            // Place sculk at the support position.
            level.sculk_set(support_pos, Block::SCULK.default_state);
            // Spread veins from the new sculk block first (vanilla order:
            // setBlock → veinSpreader.spreadAll → discharge surrounding veins).
            Self::spread_all(level, support_pos);
            // Then discharge the surrounding veins.
            let skip = support.opposite();
            for vein_dir in BlockDirection::all() {
                if vein_dir == skip {
                    continue;
                }
                let vein_pos = support_pos.offset(vein_dir.to_offset());
                if let Some(vs) = level.sculk_get(vein_pos)
                    && vs.to_block_id() == BlockId::SCULK_VEIN
                {
                    Self::on_discharged(level, vein_pos);
                }
            }
            return true;
        }
        false
    }

    /// Vanilla `SculkVeinBlock.attemptSpreadVein` default behaviour.
    pub fn attempt_spread_vein(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        state: Option<BlockStateId>,
        faces: Option<u8>,
    ) -> bool {
        match faces {
            // Vanilla: `facings == null` → the same-space spreader
            // (SAME_POSITION only).
            None => Self::spread_same_space(level, pos),
            // Vanilla: empty (non-null) facings → the interface default → the
            // multiface spreader with the full DEFAULT_SPREAD_ORDER.
            Some(0) => Self::spread_all(level, pos),
            // Vanilla: non-empty facings → `regrow`, gated on the block being
            // air or holding water.
            Some(faces_bits) => {
                let Some(existing) = state.or_else(|| level.sculk_get(pos)) else {
                    return false;
                };
                if !Self::state_is_air_or_water(level, existing, pos) {
                    return false;
                }
                let faces: Vec<BlockDirection> = BlockDirection::all()
                    .into_iter()
                    .filter(|dir| faces_bits & (1 << dir.to_index()) != 0)
                    .collect();
                Self::regrow(level, pos, &faces)
            }
        }
    }

    /// Vanilla `SculkVeinBlock.regrow`.
    pub fn regrow(level: &mut dyn SculkLevel, pos: BlockPos, faces: &[BlockDirection]) -> bool {
        let mut has_any = false;
        // Vanilla always starts from a fresh sculk_vein default state,
        // discarding any stale face properties of a previous state.
        let mut new_state = Block::SCULK_VEIN.default_state.id;
        for face in faces {
            if Self::can_attach_to(level, pos, *face) {
                new_state = Self::with_face(new_state, *face, true);
                has_any = true;
            }
        }
        if !has_any {
            return false;
        }
        // Preserve waterlogging from the existing block's fluid state.
        if let Some(existing) = level.sculk_get(pos)
            && Self::state_has_water(level, existing, pos)
        {
            new_state = Self::with_waterlogged(new_state, true);
        }
        level.sculk_set(pos, new_state.to_state());
        true
    }

    /// Vanilla `SculkVeinBlock.hasSubstrateAccess`.
    pub fn has_substrate_access(
        level: &dyn SculkLevel,
        state: BlockStateId,
        pos: BlockPos,
    ) -> bool {
        if state.to_block_id() != BlockId::SCULK_VEIN {
            return false;
        }
        BlockDirection::all().into_iter().any(|dir| {
            if !Self::has_face(state, dir) {
                return false;
            }
            let neighbour = pos.offset(dir.to_offset());
            level
                .sculk_get(neighbour)
                .is_some_and(|s| is_sculk_replaceable(s.to_block_id()))
        })
    }

    /// Vanilla `onDischarged` for sculk vein.
    pub fn on_discharged(level: &mut dyn SculkLevel, pos: BlockPos) {
        let Some(state) = level.sculk_get(pos) else {
            return;
        };
        if state.to_block_id() != BlockId::SCULK_VEIN {
            return;
        }
        let mut new_state = state;
        // Vanilla removes faces whose neighbour became sculk, preserving
        // faces attached to non-sculk supports.
        for dir in BlockDirection::all() {
            if Self::has_face(new_state, dir) {
                let neighbour = pos.offset(dir.to_offset());
                let is_sculk = level
                    .sculk_get(neighbour)
                    .is_some_and(|ns| ns.to_block_id() == BlockId::SCULK);
                if is_sculk {
                    new_state = Self::with_face(new_state, dir, false);
                }
            }
        }
        // If no faces remain, replace with air (or water).
        if !Self::has_any_face(new_state) {
            new_state = if level.sculk_is_water(pos) {
                Block::WATER.default_state.id
            } else {
                Block::AIR.default_state.id
            };
        }
        level.sculk_set(pos, new_state.to_state());
    }

    /// Vanilla `MultifaceSpreader.spreadAll` — spreads from all faces.
    pub fn spread_all(level: &mut dyn SculkLevel, pos: BlockPos) -> bool {
        Self::spread_with_types(level, pos, &DEFAULT_SPREAD_ORDER)
    }

    /// Vanilla `getSameSpaceSpreader().spreadAll` — only `SAME_POSITION`
    /// spreads are attempted.
    fn spread_same_space(level: &mut dyn SculkLevel, pos: BlockPos) -> bool {
        Self::spread_with_types(level, pos, &[SpreadType::SamePosition])
    }

    fn spread_with_types(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        spread_types: &[SpreadType],
    ) -> bool {
        let mut any = false;
        for face in BlockDirection::all() {
            if Self::can_spread_from_face(level, pos, face)
                && Self::spread_from_face(level, pos, face, spread_types)
            {
                any = true;
            }
        }
        any
    }

    fn can_spread_from_face(level: &dyn SculkLevel, pos: BlockPos, face: BlockDirection) -> bool {
        let Some(state) = level.sculk_get(pos) else {
            return false;
        };
        let id = state.to_block_id();
        // Must have the face or be a non-vein block.
        if id == BlockId::SCULK_VEIN {
            Self::has_face(state, face)
        } else {
            true
        }
    }

    fn spread_from_face(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        from_face: BlockDirection,
        spread_types: &[SpreadType],
    ) -> bool {
        // Vanilla iterates directions in the outer loop and spread types in
        // the inner loop (`MultifaceSpreader.getSpreadFromFaceTowardDirection`),
        // placing at most one vein per (face, direction) pair.
        let source_state = level.sculk_get(pos);
        let source_id = source_state.map_or(BlockId::AIR, BlockStateId::to_block_id);
        let is_vein = source_id == BlockId::SCULK_VEIN;
        let mut any = false;

        for spread_dir in BlockDirection::all() {
            if spread_dir.to_axis() == from_face.to_axis() {
                continue;
            }
            // Vanilla: for sculk-vein sources, the spread direction must not
            // already have a face set.
            if is_vein {
                let Some(state) = source_state else {
                    continue;
                };
                if Self::has_face(state, spread_dir) {
                    continue;
                }
            }
            for spread_type in spread_types {
                let (target_pos, target_face) = spread_type.spread_pos(pos, spread_dir, from_face);
                if Self::can_spread_into(level, pos, target_pos, target_face) {
                    let old_state = level.sculk_get(target_pos);
                    if let Some(placed) =
                        Self::get_state_for_placement(level, target_pos, target_face, old_state)
                    {
                        level.sculk_set(target_pos, placed);
                        any = true;
                        break;
                    }
                }
            }
        }
        any
    }

    /// Vanilla `stateCanBeReplaced` for sculk vein.
    fn can_spread_into(
        level: &dyn SculkLevel,
        source_pos: BlockPos,
        placement_pos: BlockPos,
        placement_face: BlockDirection,
    ) -> bool {
        let Some(existing) = level.sculk_get(placement_pos) else {
            return false;
        };
        let existing_id = existing.to_block_id();
        // Vanilla rejects when the SUPPORT block behind the placement face is
        // sculk / sculk_catalyst / moving_piston.
        let against_pos = placement_pos.offset(placement_face.to_offset());
        if let Some(against) = level.sculk_get(against_pos) {
            let against_id = against.to_block_id();
            if against_id == BlockId::SCULK
                || against_id == BlockId::SCULK_CATALYST
                || against_id == BlockId::MOVING_PISTON
            {
                return false;
            }
        }
        // Manhattan distance 2 check.
        let manhattan = (placement_pos.0.x - source_pos.0.x).abs()
            + (placement_pos.0.y - source_pos.0.y).abs()
            + (placement_pos.0.z - source_pos.0.z).abs();
        if manhattan == 2 {
            let neighbor_pos = source_pos.offset(placement_face.opposite().to_offset());
            if level.sculk_is_face_sturdy(neighbor_pos, placement_face) {
                return false;
            }
        }
        // Fire check.
        if existing_id == Block::FIRE.id {
            return false;
        }
        // Non-water fluids can't be replaced (vanilla checks the existing
        // fluid state is empty or water).
        if existing.to_state().is_liquid()
            && !(existing_id == BlockId::WATER && level.sculk_is_water(placement_pos))
        {
            return false;
        }
        // Vanilla: `existingState.canBeReplaced() || super.stateCanBeReplaced()`
        // where the default accepts air, the same multiface block, or a water
        // source.
        existing.to_state().replaceable()
            || existing_id == BlockId::AIR
            || existing_id == BlockId::SCULK_VEIN
            || (existing_id == BlockId::WATER && level.sculk_is_water_source(placement_pos))
    }

    fn get_state_for_placement(
        level: &dyn SculkLevel,
        placement_pos: BlockPos,
        face: BlockDirection,
        old_state: Option<BlockStateId>,
    ) -> Option<&'static BlockState> {
        // Vanilla `MultifaceBlock.isValidStateForPlacement`: the support
        // behind the face must be sturdy and the face must not already be set.
        if !Self::can_attach_to(level, placement_pos, face) {
            return None;
        }
        if let Some(old) = old_state
            && old.to_block_id() == BlockId::SCULK_VEIN
            && Self::has_face(old, face)
        {
            return None;
        }
        // Determine base state: if already sculk_vein, extend it;
        // otherwise start from a fresh sculk_vein default state.
        let mut base = old_state.map_or(Block::SCULK_VEIN.default_state.id, |s| {
            if s.to_block_id() == BlockId::SCULK_VEIN {
                s
            } else {
                Block::SCULK_VEIN.default_state.id
            }
        });
        base = Self::with_face(base, face, true);
        // Vanilla: `oldState.getFluidState().isSourceOfType(Fluids.WATER)`.
        if let Some(s) = old_state
            && Self::old_state_is_water_source(level, s, placement_pos)
        {
            base = Self::with_waterlogged(base, true);
        }
        Some(base.to_state())
    }

    /// Vanilla `MultifaceBlock.canAttachTo` — checks whether the face of the
    /// support block is sturdy (collision shape full).
    fn can_attach_to(level: &dyn SculkLevel, pos: BlockPos, face: BlockDirection) -> bool {
        let support_pos = pos.offset(face.to_offset());
        level.sculk_is_face_sturdy(support_pos, face.opposite())
    }

    /// Vanilla `fluidState.is(Fluids.WATER)` for the given state.
    fn state_has_water(level: &dyn SculkLevel, state: BlockStateId, pos: BlockPos) -> bool {
        match state.to_block_id() {
            BlockId::WATER => level.sculk_is_water(pos),
            BlockId::SCULK_VEIN => {
                let props = GlowLichenLikeProperties::from_state_id(state, &Block::SCULK_VEIN);
                props.r#waterlogged
            }
            _ => false,
        }
    }

    /// Vanilla `state.isAir() || state.getFluidState().is(Fluids.WATER)` —
    /// the gate before `regrow`.
    fn state_is_air_or_water(level: &dyn SculkLevel, state: BlockStateId, pos: BlockPos) -> bool {
        state.to_state().is_air() || Self::state_has_water(level, state, pos)
    }

    /// Vanilla `oldState.getFluidState().isSourceOfType(Fluids.WATER)`.
    fn old_state_is_water_source(
        level: &dyn SculkLevel,
        state: BlockStateId,
        pos: BlockPos,
    ) -> bool {
        match state.to_block_id() {
            BlockId::WATER => level.sculk_is_water_source(pos),
            BlockId::SCULK_VEIN => {
                let props = GlowLichenLikeProperties::from_state_id(state, &Block::SCULK_VEIN);
                props.r#waterlogged
            }
            _ => false,
        }
    }

    /// Returns whether `state` has the given face bit set.
    pub fn has_face(state: BlockStateId, face: BlockDirection) -> bool {
        if state.to_block_id() != BlockId::SCULK_VEIN {
            return false;
        }
        let props = GlowLichenLikeProperties::from_state_id(state, &Block::SCULK_VEIN);
        match face {
            BlockDirection::Down => props.r#down,
            BlockDirection::Up => props.r#up,
            BlockDirection::North => props.r#north,
            BlockDirection::South => props.r#south,
            BlockDirection::West => props.r#west,
            BlockDirection::East => props.r#east,
        }
    }

    /// Returns whether any face bit is set.
    pub fn has_any_face(state: BlockStateId) -> bool {
        BlockDirection::all()
            .into_iter()
            .any(|dir| Self::has_face(state, dir))
    }

    /// Returns a new state with the given face set to `value`.
    pub fn with_face(state: BlockStateId, face: BlockDirection, value: bool) -> BlockStateId {
        let mut props = GlowLichenLikeProperties::from_state_id(state, &Block::SCULK_VEIN);
        match face {
            BlockDirection::Down => props.r#down = value,
            BlockDirection::Up => props.r#up = value,
            BlockDirection::North => props.r#north = value,
            BlockDirection::South => props.r#south = value,
            BlockDirection::West => props.r#west = value,
            BlockDirection::East => props.r#east = value,
        }
        BlockState::from_id(props.to_state_id(&Block::SCULK_VEIN)).id
    }

    fn with_waterlogged(state: BlockStateId, value: bool) -> BlockStateId {
        let mut props = GlowLichenLikeProperties::from_state_id(state, &Block::SCULK_VEIN);
        props.r#waterlogged = value;
        BlockState::from_id(props.to_state_id(&Block::SCULK_VEIN)).id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn spread_type_same_position() {
        let pos = BlockPos::new(10, 60, 10);
        let (p, f) =
            SpreadType::SamePosition.spread_pos(pos, BlockDirection::Up, BlockDirection::North);
        assert_eq!(p, pos);
        assert_eq!(f, BlockDirection::Up);
    }

    #[test]
    fn spread_type_same_plane() {
        let pos = BlockPos::new(10, 60, 10);
        let (p, f) =
            SpreadType::SamePlane.spread_pos(pos, BlockDirection::East, BlockDirection::North);
        assert_eq!(p.0, Vector3::new(11, 60, 10));
        assert_eq!(f, BlockDirection::North);
    }

    #[test]
    fn spread_type_wrap_around() {
        let pos = BlockPos::new(10, 60, 10);
        let (p, f) =
            SpreadType::WrapAround.spread_pos(pos, BlockDirection::East, BlockDirection::North);
        assert_eq!(p.0, Vector3::new(11, 60, 9));
        assert_eq!(f, BlockDirection::West);
    }

    #[test]
    fn has_face_bit_operations() {
        let base = Block::SCULK_VEIN.default_state.id;
        assert!(!VeinRules::has_any_face(base));
        let with_up = VeinRules::with_face(base, BlockDirection::Up, true);
        assert!(VeinRules::has_face(with_up, BlockDirection::Up));
        assert!(!VeinRules::has_face(with_up, BlockDirection::Down));
        let removed = VeinRules::with_face(with_up, BlockDirection::Up, false);
        assert!(!VeinRules::has_any_face(removed));
    }
}
