//! Vanilla-faithful sculk-vein spreading logic.
//!
//! Reference: `net.minecraft.world.level.block.SculkVeinBlock` and
//! `net.minecraft.world.level.block.MultifaceSpreader` (mc-26_2).

use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::GlowLichenLikeProperties;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::SculkLevel;
use super::is_sculk_behaviour;
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
                pos.offset(spread_direction.to_offset()).offset(from_face.to_offset()),
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
    #[allow(dead_code)]
    pub fn attempt_place_sculk(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        replaceable: impl Fn(BlockId) -> bool,
    ) -> bool {
        let Some(state) = level.sculk_get(pos) else {
            return false;
        };

        for support in BlockDirection::all() {
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
            // Spread veins from the new sculk block.
            let skip = support.opposite();
            for vein_dir in BlockDirection::all() {
                if vein_dir == skip {
                    continue;
                }
                let vein_pos = support_pos.offset(vein_dir.to_offset());
                if let Some(vs) = level.sculk_get(vein_pos) {
                    if vs.to_block_id() == BlockId::SCULK_VEIN {
                        Self::on_discharged(level, vein_pos);
                    }
                }
            }
            // Also spread veins from the new sculk (via spreadAll).
            Self::spread_all(level, support_pos);
            return true;
        }
        false
    }

    /// Vanilla `SculkVeinBlock.attemptSpreadVein` default behaviour.
    pub fn attempt_spread_vein(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        _state: Option<BlockStateId>,
        faces: &[BlockDirection],
    ) -> bool {
        if faces.is_empty() {
            // No faces — same-space spreader.
            return Self::spread_all(level, pos);
        }
        // Has faces — regrow.
        Self::regrow(level, pos, faces)
    }

    /// Vanilla `SculkVeinBlock.regrow`.
    pub fn regrow(
        level: &mut dyn SculkLevel,
        pos: BlockPos,
        faces: &[BlockDirection],
    ) -> bool {
        let mut has_any = false;
        // Initialize from the existing state at pos to preserve waterlogged
        // and already-enabled faces; fall back to a fresh sculk_vein default.
        let mut new_state = match level.sculk_get(pos) {
            Some(s) if s.to_block_id() == BlockId::SCULK_VEIN => s,
            Some(s) if s.to_block_id() == BlockId::WATER => {
                Self::with_waterlogged(Block::SCULK_VEIN.default_state.id, true)
            }
            _ => Block::SCULK_VEIN.default_state.id,
        };
        for face in faces {
            if Self::can_attach_to(level, pos, *face) {
                new_state = Self::with_face(new_state, *face, true);
                has_any = true;
            }
        }
        if !has_any {
            return false;
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
        // Remove faces that point to non-sculk neighbours.
        for dir in BlockDirection::all() {
            if Self::has_face(new_state, dir) {
                let neighbour = pos.offset(dir.to_offset());
                // If the neighbour is unavailable (out of bounds) or is not
                // sculk, clear the face. This prevents veins from hanging
                // onto non-existent or non-sculk blocks.
                let is_sculk = level
                    .sculk_get(neighbour)
                    .is_some_and(|ns| ns.to_block_id() == BlockId::SCULK);
                if !is_sculk {
                    new_state = Self::with_face(new_state, dir, false);
                }
            }
        }
        // If no faces remain, replace with air (or water).
        if !Self::has_any_face(new_state) {
            // Check if water should go here.
            new_state = if level.sculk_is_water_source(pos) {
                Block::WATER.default_state.id
            } else {
                Block::AIR.default_state.id
            };
        }
        level.sculk_set(pos, new_state.to_state());
    }

    /// Vanilla `MultifaceSpreader.spreadAll` — spreads from all faces.
    pub fn spread_all(level: &mut dyn SculkLevel, pos: BlockPos) -> bool {
        let mut any = false;
        for face in BlockDirection::all() {
            if Self::can_spread_from_face(level, pos, face)
                && Self::spread_from_face(level, pos, face)
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
    ) -> bool {
        for spread_type in DEFAULT_SPREAD_ORDER {
            for spread_dir in BlockDirection::all() {
                if spread_dir.to_axis() == from_face.to_axis() {
                    continue;
                }
                let (target_pos, target_face) = spread_type.spread_pos(pos, spread_dir, from_face);
                if Self::can_spread_into(level, pos, target_pos, target_face) {
                    let old_state = level.sculk_get(target_pos);
                    if let Some(placed) =
                        Self::get_state_for_placement(level, target_pos, target_face, old_state)
                    {
                        level.sculk_set(target_pos, placed);
                        return true;
                    }
                }
            }
        }
        false
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
        // Can't spread into sculk, sculk_catalyst, moving_piston.
        if existing_id == BlockId::SCULK
            || existing_id == BlockId::SCULK_CATALYST
            || existing_id == BlockId::MOVING_PISTON
        {
            return false;
        }
        // Manhattan distance 2 check.
        let manhattan = (placement_pos.0.x - source_pos.0.x).abs()
            + (placement_pos.0.y - source_pos.0.y).abs()
            + (placement_pos.0.z - source_pos.0.z).abs();
        if manhattan == 2 {
            let neighour_pos = source_pos.offset(placement_face.opposite().to_offset());
            if level.sculk_is_face_sturdy(neighour_pos, placement_face) {
                return false;
            }
        }
        // Fire check.
        if existing_id == Block::FIRE.id {
            return false;
        }
        // Replaceable check: air, water source, or sculk_vein.
        existing_id == BlockId::AIR
            || existing_id == BlockId::SCULK_VEIN
            || (existing_id == BlockId::WATER && level.sculk_is_water_source(placement_pos))
            || is_sculk_replaceable(existing_id)
    }

    fn get_state_for_placement(
        _level: &dyn SculkLevel,
        _placement_pos: BlockPos,
        face: BlockDirection,
        old_state: Option<BlockStateId>,
    ) -> Option<&'static BlockState> {
        // Determine base state: if already sculk_vein, extend it;
        // otherwise start from a fresh sculk_vein default state.
        let mut base = if let Some(s) = old_state {
            if s.to_block_id() == BlockId::SCULK_VEIN {
                s
            } else {
                Block::SCULK_VEIN.default_state.id
            }
        } else {
            Block::SCULK_VEIN.default_state.id
        };
        base = Self::with_face(base, face, true);
        // Preserve waterlogging.
        if let Some(s) = old_state {
            if s.to_block_id() == BlockId::WATER {
                base = Self::with_waterlogged(base, true);
            }
        }
        Some(base.to_state())
    }

    /// Vanilla `MultifaceBlock.canAttachTo` — checks whether the face of the
    /// support block is sturdy (collision shape full).
    fn can_attach_to(level: &dyn SculkLevel, pos: BlockPos, face: BlockDirection) -> bool {
        let support_pos = pos.offset(face.to_offset());
        level.sculk_is_face_sturdy(support_pos, face.opposite())
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
        BlockDirection::all().into_iter().any(|dir| Self::has_face(state, dir))
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

    #[test]
    fn spread_type_same_position() {
        let pos = BlockPos::new(10, 60, 10);
        let (p, f) = SpreadType::SamePosition.spread_pos(pos, BlockDirection::Up, BlockDirection::North);
        assert_eq!(p, pos);
        assert_eq!(f, BlockDirection::Up);
    }

    #[test]
    fn spread_type_same_plane() {
        let pos = BlockPos::new(10, 60, 10);
        let (p, f) = SpreadType::SamePlane.spread_pos(pos, BlockDirection::East, BlockDirection::North);
        assert_eq!(p.0, Vector3::new(11, 60, 10));
        assert_eq!(f, BlockDirection::North);
    }

    #[test]
    fn spread_type_wrap_around() {
        let pos = BlockPos::new(10, 60, 10);
        let (p, f) = SpreadType::WrapAround.spread_pos(pos, BlockDirection::East, BlockDirection::North);
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
