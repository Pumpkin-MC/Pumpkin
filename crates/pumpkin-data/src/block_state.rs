use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use crate::block_properties::NoteblockInstrument;
use crate::block_registry::collision_shape;
use crate::{Block, BlockDirection, BlockId};

/// Represents a specific state of a block, including its properties and physical behaviors.
///
/// A single `Block` (like a Hopper) can have multiple `BlockState`s (e.g., pointing North,
/// South, or being powered). This struct is optimized for high-speed lookups during
/// physics and lighting calculations.
#[derive(Debug)]
pub struct BlockState {
    /// The global palette ID used for network serialization and chunk storage.
    pub id: BlockStateId,
    /// Bit-flags representing boolean or enum properties (e.g., `waterlogged`, `lit`, `facing`).
    pub state_flags: u16,
    /// Cached flags for each of the 6 sides to speed up ambient occlusion and face culling.
    pub side_flags: u8,
    /// The note block instrument produced when this block is placed underneath one.
    pub instrument: NoteblockInstrument,
    /// The light level emitted by this block, ranging from 0 to 15.
    pub luminance: u8,
    /// Defines how the block reacts to being pushed or pulled by a piston.
    pub piston_behavior: PistonBehavior,
    /// Overrides the base block hardness for this specific state if necessary.
    pub hardness: f32,
    /// Indices into a global voxel-shape registry for physical entity collisions.
    pub collision_shapes: &'static [u16],
    /// Indices into a global voxel-shape registry for the selection highlight box.
    pub outline_shapes: &'static [u16],
    /// How much light is subtracted as it passes through this block (0 for transparent, 15 for opaque).
    pub opacity: u8,
    /// The ID of the block entity associated with this state.
    /// Set to `u16::MAX` if the block does not hold NBT data.
    pub block_entity_type: u16,
}

/// Helper struct to ensure the validity of `BlockStateIds` parsed from external sources.
/// Every [`BlockStateId`] is guaranteed to correspond to a valid [`BlockState`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BlockStateId(u16);

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PistonBehavior {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

impl PartialEq<BlockStateId> for BlockState {
    fn eq(&self, other: &BlockStateId) -> bool {
        self.id == *other
    }
}

impl PartialEq<BlockState> for BlockStateId {
    fn eq(&self, other: &BlockState) -> bool {
        *self == other.id
    }
}

impl PartialEq for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BlockState {}

impl BlockState {
    #[must_use]
    pub const fn is_air(&self) -> bool {
        self.state_flags & IS_AIR != 0
    }

    #[must_use]
    pub const fn burnable(&self) -> bool {
        self.state_flags & BURNABLE != 0
    }

    #[must_use]
    pub const fn tool_required(&self) -> bool {
        self.state_flags & TOOL_REQUIRED != 0
    }

    #[must_use]
    pub const fn sided_transparency(&self) -> bool {
        self.state_flags & SIDED_TRANSPARENCY != 0
    }

    #[must_use]
    pub const fn replaceable(&self) -> bool {
        self.state_flags & REPLACEABLE != 0
    }

    #[must_use]
    pub const fn is_liquid(&self) -> bool {
        self.state_flags & IS_LIQUID != 0
    }

    /// Returns the legacy value for whether a block is solid.
    #[must_use]
    pub const fn is_solid(&self) -> bool {
        self.state_flags & IS_SOLID != 0
    }

    #[must_use]
    pub const fn is_full_cube(&self) -> bool {
        self.state_flags & IS_FULL_CUBE != 0
    }

    /// Returns whether the block is solid.
    /// Solid blocks conduct redstone and block redstone wire.
    /// Non-solid blocks don't allow redstone wire on top to propagate their signal downwards in java.
    #[must_use]
    pub const fn is_solid_block(&self) -> bool {
        self.state_flags & IS_SOLID_BLOCK != 0
    }

    #[must_use]
    pub const fn has_random_ticks(&self) -> bool {
        self.state_flags & HAS_RANDOM_TICKS != 0
    }

    /// Returns whether this block state is a full opaque cube (`isSolidRender()` in Java).
    #[must_use]
    pub const fn is_solid_render(&self) -> bool {
        self.state_flags & IS_SOLID_RENDER != 0
    }

    /// Returns whether this block state can occlude other blocks (`canOcclude` in Java).
    #[must_use]
    pub const fn can_occlude(&self) -> bool {
        self.state_flags & CAN_OCCLUDE != 0
    }

    /// Returns whether this block produces an analog output signal for comparators (`hasAnalogOutputSignal()` in Java).
    #[must_use]
    pub const fn has_analog_output_signal(&self) -> bool {
        self.state_flags & HAS_ANALOG_OUTPUT_SIGNAL != 0
    }

    /// Whether an entity whose head is inside this block is spared from suffocating. Vanilla
    /// blocks follow the default (a full block that blocks movement suffocates), only blocks that
    /// plugins register can opt out, like vanilla's `isSuffocating((state, level, pos) -> false)`.
    #[must_use]
    pub const fn never_suffocates(&self) -> bool {
        self.state_flags & NEVER_SUFFOCATES != 0
    }

    ///`isFaceSturdy()` in Java!
    #[must_use]
    pub const fn is_side_solid(&self, side: BlockDirection) -> bool {
        match side {
            BlockDirection::Down => self.side_flags & DOWN_SIDE_SOLID != 0,
            BlockDirection::Up => self.side_flags & UP_SIDE_SOLID != 0,
            BlockDirection::North => self.side_flags & NORTH_SIDE_SOLID != 0,
            BlockDirection::South => self.side_flags & SOUTH_SIDE_SOLID != 0,
            BlockDirection::West => self.side_flags & WEST_SIDE_SOLID != 0,
            BlockDirection::East => self.side_flags & EAST_SIDE_SOLID != 0,
        }
    }

    ///isSideSolid(..., Direction.UP, SideShapeType.CENTER) in Java!
    ///Only valid for UP and DOWN sides
    #[must_use]
    pub const fn is_center_solid(&self, side: BlockDirection) -> bool {
        match side {
            BlockDirection::Down => self.side_flags & DOWN_CENTER_SOLID != 0,
            BlockDirection::Up => self.side_flags & UP_CENTER_SOLID != 0,
            _ => false,
        }
    }

    #[must_use]
    pub fn is_waterlogged(&self) -> bool {
        self.id.is_waterlogged()
    }

    /// Produce a new state identical to `self` except the waterlogged property
    /// is set to `value`. If the block type does not support waterlogging or
    /// the state already had waterlogged set to `value`, `None` is returned.
    #[must_use]
    pub fn set_waterlogged(&self, value: bool) -> Option<&'static BlockState> {
        self.id
            .to_block()
            .set_waterlogged(self.id, value)
            .map(BlockStateId::to_state)
    }

    pub fn get_block_collision_shapes(&self) -> impl Iterator<Item = BoundingBox> + '_ {
        self.collision_shapes.iter().map(|&id| collision_shape(id))
    }

    /// Returns block-local collision shapes with vanilla's coordinate-derived offset applied.
    pub fn get_block_collision_shapes_at(
        &self,
        pos: &BlockPos,
    ) -> impl Iterator<Item = BoundingBox> + '_ {
        let offset = Block::from_state_id(self.id).shape_offset_delta(pos);
        self.get_block_collision_shapes()
            .map(move |shape| shape.shift(offset))
    }

    pub fn get_block_outline_shapes(&self) -> impl Iterator<Item = BoundingBox> + '_ {
        let base_shapes = self.outline_shapes.iter().map(|&id| collision_shape(id));

        let water_shape = self
            .is_waterlogged()
            .then(|| BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.875, 1.0)));

        base_shapes.chain(water_shape)
    }

    /// Returns block-local outline shapes with vanilla's coordinate-derived offset applied.
    pub fn get_block_outline_shapes_at(
        &self,
        pos: &BlockPos,
    ) -> impl Iterator<Item = BoundingBox> + '_ {
        let offset = Block::from_state_id(self.id).shape_offset_delta(pos);
        let base_shapes = self
            .outline_shapes
            .iter()
            .map(move |&id| collision_shape(id).shift(offset));

        let water_shape = self
            .is_waterlogged()
            .then(|| BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.875, 1.0)));

        base_shapes.chain(water_shape)
    }

    #[must_use]
    pub fn rotate(&self, rotation: crate::block_rotation::Rotation) -> &'static Self {
        Block::from_state_id(self.id).rotate(self.id, rotation)
    }

    #[must_use]
    pub fn mirror(&self, mirror: crate::block_rotation::Mirror) -> &'static Self {
        Block::from_state_id(self.id).mirror(self.id, mirror)
    }
}

impl BlockStateId {
    // depends on generated impl:
    // pub(crate) const STATE_COUNT: u16;

    /// The count of the generated (vanilla) block states. The states of blocks that plugins
    /// register get the ids after these, see [`Self::total_count`].
    pub const COUNT: u16 = Self::STATE_COUNT;

    // SAFETY: There must never be a BlockStateId that is neither a generated state
    // (< STATE_COUNT) nor a state of a registered dynamic block (checked by `from_raw`). The
    // generated lookups rely on it.

    /// A generated state id. Const, so it cannot see registered blocks, use [`Self::from_raw`] for
    /// ids from plugins, the network or disk.
    #[inline]
    #[must_use]
    pub const fn new(inner: u16) -> Option<Self> {
        if inner < Self::STATE_COUNT {
            return Some(Self(inner));
        }
        None
    }

    /// A generated state id, air for anything else. See [`Self::new`].
    #[inline]
    #[must_use]
    pub const fn new_or_air(inner: u16) -> Self {
        if inner < Self::STATE_COUNT {
            return Self(inner);
        }
        Self::AIR
    }

    /// The id of a state of a registered dynamic block. Only the dynamic registry may call this,
    /// with an id it handed out.
    #[inline]
    #[must_use]
    pub(crate) const fn from_dynamic_raw(inner: u16) -> Self {
        Self(inner)
    }

    /// Number of generated and registered block states. Ids are `0..total_count()`, so this is
    /// what the size of the global palette on the network derives from.
    #[inline]
    #[must_use]
    pub fn total_count() -> u32 {
        crate::block_registry::total_state_count()
    }

    /// A state id of a generated or registered block.
    #[inline]
    #[must_use]
    pub fn from_raw(inner: u16) -> Option<Self> {
        (u32::from(inner) < Self::total_count()).then_some(Self(inner))
    }

    /// Like [`Self::from_raw`], air for an id that does not exist.
    #[inline]
    #[must_use]
    pub fn from_raw_or_air(inner: u16) -> Self {
        Self::from_raw(inner).unwrap_or(Self::AIR)
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn to_state(self) -> &'static BlockState {
        BlockState::from_id(self)
    }

    #[inline]
    #[must_use]
    pub fn to_block_id(self) -> BlockId {
        BlockId::from_state_id(self)
    }

    #[inline]
    #[must_use]
    pub fn to_block(self) -> &'static Block {
        Block::from_state_id(self)
    }

    #[inline]
    #[must_use]
    pub fn is_solid_render(self) -> bool {
        self.to_state().is_solid_render()
    }

    #[inline]
    #[must_use]
    pub fn can_occlude(self) -> bool {
        self.to_state().can_occlude()
    }

    #[inline]
    #[must_use]
    pub fn has_analog_output_signal(self) -> bool {
        self.to_state().has_analog_output_signal()
    }

    #[inline]
    #[must_use]
    pub fn is_waterlogged(self) -> bool {
        self.to_block().is_waterlogged(self)
    }

    #[inline]
    #[must_use]
    pub fn rotate(self, rotation: crate::block_rotation::Rotation) -> &'static BlockState {
        Block::from_state_id(self).rotate(self, rotation)
    }

    #[inline]
    #[must_use]
    pub fn mirror(self, mirror: crate::block_rotation::Mirror) -> &'static BlockState {
        Block::from_state_id(self).mirror(self, mirror)
    }
}

impl Default for BlockStateId {
    #[inline]
    fn default() -> Self {
        Self::AIR
    }
}

impl std::fmt::Display for BlockStateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlockStateId({} = \"{}\")",
            self.0,
            Block::from_state_id(*self).name
        )
    }
}

//This is the Layout of state_props in the right order
// state_flags
pub(crate) const IS_AIR: u16 = 1 << 0;
pub(crate) const BURNABLE: u16 = 1 << 1;
pub(crate) const TOOL_REQUIRED: u16 = 1 << 2;
pub(crate) const SIDED_TRANSPARENCY: u16 = 1 << 3;
pub(crate) const REPLACEABLE: u16 = 1 << 4;
pub(crate) const IS_LIQUID: u16 = 1 << 5;
pub(crate) const IS_SOLID: u16 = 1 << 6;
pub(crate) const IS_FULL_CUBE: u16 = 1 << 7;
pub(crate) const IS_SOLID_BLOCK: u16 = 1 << 8;
pub(crate) const HAS_RANDOM_TICKS: u16 = 1 << 9;
pub(crate) const IS_SOLID_RENDER: u16 = 1 << 10;
pub(crate) const CAN_OCCLUDE: u16 = 1 << 11;
pub(crate) const HAS_ANALOG_OUTPUT_SIGNAL: u16 = 1 << 12;
/// Only set on blocks that plugins register, see `BlockState::never_suffocates`.
pub(crate) const NEVER_SUFFOCATES: u16 = 1 << 13;

// side_flags
pub(crate) const DOWN_SIDE_SOLID: u8 = 1 << 0;
pub(crate) const UP_SIDE_SOLID: u8 = 1 << 1;
pub(crate) const NORTH_SIDE_SOLID: u8 = 1 << 2;
pub(crate) const SOUTH_SIDE_SOLID: u8 = 1 << 3;
pub(crate) const WEST_SIDE_SOLID: u8 = 1 << 4;
pub(crate) const EAST_SIDE_SOLID: u8 = 1 << 5;
pub(crate) const DOWN_CENTER_SOLID: u8 = 1 << 6;
pub(crate) const UP_CENTER_SOLID: u8 = 1 << 7;

#[cfg(test)]
mod tests {
    use crate::Block;
    use pumpkin_util::math::position::BlockPos;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
    }

    #[test]
    fn bamboo_collision_shape_uses_its_world_position() {
        let state = Block::BAMBOO.default_state;
        let origin_shape = state
            .get_block_collision_shapes_at(&BlockPos::new(0, 64, 0))
            .next()
            .unwrap();
        let shifted_shape = state
            .get_block_collision_shapes_at(&BlockPos::new(-18, 64, -7))
            .next()
            .unwrap();

        assert_close(origin_shape.min.x, 0.15625);
        assert_close(origin_shape.max.x, 0.34375);
        assert_close(shifted_shape.min.x, 0.65625);
        assert_close(shifted_shape.max.x, 0.84375);
        assert_close(shifted_shape.min.z, 0.65625);
        assert_close(shifted_shape.max.z, 0.84375);
    }
}
