//! Sculk generation — faithful reimplementation of the vanilla Minecraft
//! `SculkSpreader` / `SculkBlock` / `SculkVeinBlock` algorithm.
//!
//! Organised into three sub-modules:
//! - [`spreader`] — cursor movement, charge decay, merging.
//! - [`vein`]     — multiface (sculk vein) spreading via support faces.
//! - [`growth`]   — sculk sensor / shrieker placement rules.

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::tag::Block::MINECRAFT_SCULK_REPLACEABLE;
use pumpkin_data::tag::Block::MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub mod growth;
pub mod spreader;
pub mod vein;

/// The 18 non-corner neighbours of a block: every offset in the 3×3×3 cube
/// that shares at least one zero-axis (i.e. is NOT a corner) and is not the
/// centre itself. This matches vanilla's `NON_CORNER_NEIGHBOURS`.
pub const NON_CORNER_NEIGHBOURS: [Vector3<i32>; 18] = [
    // Face-adjacent — 6
    Vector3::new(-1, 0, 0),
    Vector3::new(1, 0, 0),
    Vector3::new(0, -1, 0),
    Vector3::new(0, 1, 0),
    Vector3::new(0, 0, -1),
    Vector3::new(0, 0, 1),
    // Edge-adjacent (same Y plane) — 4
    Vector3::new(-1, 0, -1),
    Vector3::new(-1, 0, 1),
    Vector3::new(1, 0, -1),
    Vector3::new(1, 0, 1),
    // Edge-adjacent (vertical X) — 4
    Vector3::new(-1, -1, 0),
    Vector3::new(-1, 1, 0),
    Vector3::new(1, -1, 0),
    Vector3::new(1, 1, 0),
    // Edge-adjacent (vertical Z) — 4
    Vector3::new(0, -1, -1),
    Vector3::new(0, -1, 1),
    Vector3::new(0, 1, -1),
    Vector3::new(0, 1, 1),
];

/// Abstraction over the level that the sculk spreader reads from and writes to.
/// Implemented for `T: GenerationCache` (post-terrain) via a blanket impl,
/// and for proto-chunk generation through [`ProtoChunkSculkView`].
pub trait SculkLevel {
    /// Returns the block state at `pos`, or `None` if the position is
    /// unwritable (out of the generating chunk). A `None` is treated as a
    /// solid obstacle by the spreader — cursors cannot move through it.
    fn sculk_get(&self, pos: BlockPos) -> Option<BlockStateId>;

    /// Sets the block state at `pos`. No-op if the position is unwritable.
    fn sculk_set(&mut self, pos: BlockPos, state: &'static BlockState);

    /// Whether the position is completely empty (air or void).
    fn sculk_is_air(&self, pos: BlockPos) -> bool;

    /// Whether the position holds water (source).
    fn sculk_is_water_source(&self, pos: BlockPos) -> bool;

    /// Whether the position holds water (any water fluid state, matching
    /// vanilla `FluidState.is(Fluids.WATER)`).
    fn sculk_is_water(&self, pos: BlockPos) -> bool;

    /// Whether the face of the block at `pos` in the given direction is
    /// sturdy (equivalent to Java `isFaceSturdy`).
    fn sculk_is_face_sturdy(&self, pos: BlockPos, face: BlockDirection) -> bool;

    /// Whether the block at `pos` is a full cube (collision-shape full block,
    /// used by `canSpreadFrom`).
    fn sculk_is_full_cube(&self, pos: BlockPos) -> bool;
}

/// Returns `true` if the block id implements vanilla `SculkBehaviour`.
///
/// Only `sculk` and `sculk_vein` implement `SculkBehaviour` in vanilla
/// (`SculkBlock` / `SculkVeinBlock`). Sensors, shriekers and catalysts are
/// plain blocks and therefore use `SculkBehaviour.DEFAULT` when a cursor
/// visits them.
pub const fn is_sculk_behaviour(id: BlockId) -> bool {
    matches!(id, BlockId::SCULK | BlockId::SCULK_VEIN)
}

/// Returns `true` if the block id is tagged `minecraft:sculk_replaceable`.
pub fn is_sculk_replaceable(id: BlockId) -> bool {
    id.has_tag(MINECRAFT_SCULK_REPLACEABLE)
}

/// Returns `true` if the block id is tagged
/// `minecraft:sculk_replaceable_world_gen`.
pub fn is_sculk_replaceable_world_gen(id: BlockId) -> bool {
    id.has_tag(MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN)
}

/// Vanilla `canSpreadFrom` — checks whether a sculk patch can originate at
/// the given position.
pub fn can_spread_from(level: &dyn SculkLevel, pos: BlockPos) -> bool {
    let Some(state) = level.sculk_get(pos) else {
        return false;
    };
    let block_id = state.to_block_id();
    if is_sculk_behaviour(block_id) {
        return true;
    }
    if !level.sculk_is_air(pos) && !level.sculk_is_water_source(pos) {
        return false;
    }
    // Position is air or water source — needs a full-cube neighbour.
    BlockDirection::all()
        .into_iter()
        .any(|dir| level.sculk_is_full_cube(pos.offset(dir.to_offset())))
}

/// Builds the shrieker block state used by the extra-rare-growths pass
/// (with `can_summon` set when world-gen).
pub fn shrieker_state(can_summon: bool) -> &'static BlockState {
    let mut properties = pumpkin_data::block_properties::SculkShriekerLikeProperties::default(
        &Block::SCULK_SHRIEKER,
    );
    properties.r#can_summon = can_summon;
    BlockState::from_id(properties.to_state_id(&Block::SCULK_SHRIEKER))
}

/// Convenience: resolve the block id at a position, defaulting to air when
/// unwritable.
pub fn sculk_block_id(level: &dyn SculkLevel, pos: BlockPos) -> BlockId {
    level
        .sculk_get(pos)
        .map_or(BlockId::AIR, BlockStateId::to_block_id)
}

// Blanket impl for any type implementing GenerationCache (post-terrain case).

use crate::generation::proto_chunk::GenerationCache;

/// Blanket implementation of `SculkLevel` for every type that implements
/// `GenerationCache`. This covers the `Cache` used during post-terrain
/// feature generation.
impl<T: GenerationCache> SculkLevel for T {
    fn sculk_get(&self, pos: BlockPos) -> Option<BlockStateId> {
        Some(GenerationCache::get_block_state(self, &pos.0))
    }

    fn sculk_set(&mut self, pos: BlockPos, state: &'static BlockState) {
        GenerationCache::set_block_state(self, &pos.0, state);
    }

    fn sculk_is_air(&self, pos: BlockPos) -> bool {
        GenerationCache::is_air(self, &pos.0)
    }

    fn sculk_is_water_source(&self, pos: BlockPos) -> bool {
        let (fluid, fluid_state) = GenerationCache::get_fluid_and_fluid_state(self, &pos.0);
        fluid.id == 2 && fluid_state.is_source
    }

    fn sculk_is_water(&self, pos: BlockPos) -> bool {
        let (fluid, _fluid_state) = GenerationCache::get_fluid_and_fluid_state(self, &pos.0);
        fluid.id == 2
    }

    fn sculk_is_face_sturdy(&self, pos: BlockPos, face: BlockDirection) -> bool {
        GenerationCache::get_block_state(self, &pos.0)
            .to_state()
            .is_side_solid(face)
    }

    fn sculk_is_full_cube(&self, pos: BlockPos) -> bool {
        GenerationCache::get_block_state(self, &pos.0)
            .to_state()
            .is_full_cube()
    }
}

// ProtoChunk wrapper for world-gen (bounds-aware) case.

use crate::ProtoChunk;

/// A bounds-checked view over a [`ProtoChunk`] that implements [`SculkLevel`].
///
/// Unlike the blanket `GenerationCache` impl (which reads any chunk in the
/// cache), this wrapper returns `None` for positions outside the proto
/// chunk's XZ bounds or height range, correctly modelling them as solid
/// obstacles for the spreader.
pub struct ProtoChunkSculkView<'a> {
    chunk: &'a mut ProtoChunk,
}

impl<'a> ProtoChunkSculkView<'a> {
    /// Creates an exclusive, bounds-aware view over the supplied
    /// [`ProtoChunk`]. Out-of-bounds positions return `None` from reads
    /// and are no-ops on writes, modelling them as solid obstacles for
    /// the spreader.
    pub const fn new(chunk: &'a mut ProtoChunk) -> Self {
        Self { chunk }
    }

    const fn in_bounds(&self, pos: BlockPos) -> bool {
        if pos.0.x >> 4 != self.chunk.x || pos.0.z >> 4 != self.chunk.z {
            return false;
        }
        let local_y = pos.0.y - self.chunk.bottom_y() as i32;
        local_y >= 0 && local_y < self.chunk.height() as i32
    }
}

impl SculkLevel for ProtoChunkSculkView<'_> {
    fn sculk_get(&self, pos: BlockPos) -> Option<BlockStateId> {
        if !self.in_bounds(pos) {
            return None;
        }
        let local_x = pos.0.x & 15;
        let local_y = pos.0.y - self.chunk.bottom_y() as i32;
        let local_z = pos.0.z & 15;
        Some(self.chunk.get_block_state_raw(local_x, local_y, local_z))
    }

    fn sculk_set(&mut self, pos: BlockPos, state: &'static BlockState) {
        if !self.in_bounds(pos) {
            return;
        }
        let local_x = pos.0.x & 15;
        let local_z = pos.0.z & 15;
        self.chunk.set_block_state(local_x, pos.0.y, local_z, state);
    }

    fn sculk_is_air(&self, pos: BlockPos) -> bool {
        self.sculk_get(pos).is_none_or(is_air)
    }

    fn sculk_is_water_source(&self, pos: BlockPos) -> bool {
        // Proto chunks have no fluid simulation; water is stored as a block
        // state. Vanilla derives the fluid from the block state during
        // world-gen, where water blocks are (by convention) sources, so a
        // WATER block state is treated as a water source.
        self.sculk_get(pos)
            .is_some_and(|s| s.to_block_id() == BlockId::WATER)
    }

    fn sculk_is_water(&self, pos: BlockPos) -> bool {
        self.sculk_get(pos)
            .is_some_and(|s| s.to_block_id() == BlockId::WATER)
    }

    fn sculk_is_face_sturdy(&self, pos: BlockPos, face: BlockDirection) -> bool {
        self.sculk_get(pos)
            .is_some_and(|s| s.to_state().is_side_solid(face))
    }

    fn sculk_is_full_cube(&self, pos: BlockPos) -> bool {
        self.sculk_get(pos)
            .is_some_and(|s| s.to_state().is_full_cube())
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::collections::HashMap;

    /// In-memory [`SculkLevel`] used by the unit tests of the sculk
    /// sub-modules. Mirrors the proto-chunk view: water is detected from the
    /// block state, and missing positions read as air.
    pub struct MockSculkLevel {
        pub(crate) blocks: HashMap<BlockPos, BlockStateId>,
    }

    impl MockSculkLevel {
        pub(crate) fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }

        pub(crate) fn set_id(&mut self, pos: BlockPos, id: BlockStateId) {
            self.blocks.insert(pos, id);
        }
    }

    impl SculkLevel for MockSculkLevel {
        fn sculk_get(&self, pos: BlockPos) -> Option<BlockStateId> {
            self.blocks.get(&pos).copied()
        }

        fn sculk_set(&mut self, pos: BlockPos, state: &'static BlockState) {
            self.blocks.insert(pos, state.id);
        }

        fn sculk_is_air(&self, pos: BlockPos) -> bool {
            self.sculk_get(pos).is_none_or(|s| s.to_state().is_air())
        }

        fn sculk_is_water_source(&self, pos: BlockPos) -> bool {
            self.sculk_get(pos)
                .is_some_and(|s| s.to_block_id() == BlockId::WATER)
        }

        fn sculk_is_water(&self, pos: BlockPos) -> bool {
            self.sculk_get(pos)
                .is_some_and(|s| s.to_block_id() == BlockId::WATER)
        }

        fn sculk_is_face_sturdy(&self, pos: BlockPos, face: BlockDirection) -> bool {
            self.sculk_get(pos)
                .is_some_and(|s| s.to_state().is_side_solid(face))
        }

        fn sculk_is_full_cube(&self, pos: BlockPos) -> bool {
            self.sculk_get(pos)
                .is_some_and(|s| s.to_state().is_full_cube())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::MockSculkLevel;

    #[test]
    fn can_spread_from_water_source_origin() {
        // A water-source origin with a full-cube neighbour is a valid spread
        // origin (regression: proto-chunk view must detect water).
        let mut level = MockSculkLevel::new();
        let origin = BlockPos::new(0, 60, 0);
        level.set_id(origin, Block::WATER.default_state.id);
        level.set_id(origin.up(), Block::STONE.default_state.id);
        assert!(can_spread_from(&level, origin));
    }

    #[test]
    fn can_spread_from_rejects_solid_origin() {
        let mut level = MockSculkLevel::new();
        let origin = BlockPos::new(0, 60, 0);
        level.set_id(origin, Block::STONE.default_state.id);
        assert!(!can_spread_from(&level, origin));
    }

    #[test]
    fn sculk_behaviour_matches_vanilla_implementations() {
        // Vanilla: only sculk and sculk vein implement `SculkBehaviour`.
        // Sensors, shriekers and catalysts use `SculkBehaviour.DEFAULT`.
        assert!(is_sculk_behaviour(BlockId::SCULK));
        assert!(is_sculk_behaviour(BlockId::SCULK_VEIN));
        assert!(!is_sculk_behaviour(BlockId::SCULK_CATALYST));
        assert!(!is_sculk_behaviour(BlockId::SCULK_SENSOR));
        assert!(!is_sculk_behaviour(BlockId::CALIBRATED_SCULK_SENSOR));
        assert!(!is_sculk_behaviour(BlockId::SCULK_SHRIEKER));
    }

    #[test]
    fn can_spread_from_rejects_sensor_origin() {
        // Vanilla `SculkPatchFeature.canSpreadFrom`: a sensor is not a
        // `SculkBehaviour` and not air/water, so spreading cannot start at it.
        let mut level = MockSculkLevel::new();
        let origin = BlockPos::new(0, 60, 0);
        level.set_id(origin, Block::SCULK_SENSOR.default_state.id);
        assert!(!can_spread_from(&level, origin));
    }
}
