use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use crate::block_properties::{COLLISION_SHAPES, Instrument};
use crate::{Block, BlockDirection};

/// Represents a specific state of a block, including its properties and physical behaviors.
///
/// A single `Block` (like a Hopper) can have multiple `BlockState`s (e.g., pointing North,
/// South, or being powered). This struct is optimized for high-speed lookups during
/// physics and lighting calculations.
#[derive(Debug)]
pub struct BlockState {
    /// The global palette ID used for network serialization and chunk storage.
    pub id: u16,
    /// Bit-flags representing boolean or enum properties (e.g., `waterlogged`, `lit`, `facing`).
    pub state_flags: u16,
    /// Cached flags for each of the 6 sides to speed up ambient occlusion and face culling.
    pub side_flags: u8,
    /// The note block instrument produced when this block is placed underneath one.
    pub instrument: Instrument,
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

impl PartialEq for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PistonBehavior {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

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

    pub const fn block_movement(&self) -> bool {
        self.id != 129 && // cobweb
        self.id != 789 && // bamboo sapling
        self.is_solid()
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

    ///`isSideSolidFullSquare()` in Java!
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
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn is_waterlogged(&self) -> bool {
        let block = Block::from_state_id(self.id);

        block.properties(self.id).is_some_and(|props| {
            props
                .to_props()
                .iter()
                .any(|(k, v)| k == &"waterlogged" && v == &"true")
        })
    }

    pub fn get_block_collision_shapes(&self) -> impl Iterator<Item = BoundingBox> + '_ {
        self.collision_shapes
            .iter()
            .map(|&id| COLLISION_SHAPES[id as usize])
    }

    pub fn get_block_outline_shapes(&self) -> impl Iterator<Item = BoundingBox> + '_ {
        let base_shapes = self
            .outline_shapes
            .iter()
            .map(|&id| COLLISION_SHAPES[id as usize]);

        let water_shape = self
            .is_waterlogged()
            .then(|| BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.875, 1.0)));

        base_shapes.chain(water_shape)
    }
}

#[derive(Clone, Debug)]
pub struct BlockStateRef {
    pub id: u16,
    pub state_idx: u16,
}

//This is the Layout of state_props in the right order
// state_flags
const IS_AIR: u16 = 1 << 0;
const BURNABLE: u16 = 1 << 1;
const TOOL_REQUIRED: u16 = 1 << 2;
const SIDED_TRANSPARENCY: u16 = 1 << 3;
const REPLACEABLE: u16 = 1 << 4;
const IS_LIQUID: u16 = 1 << 5;
const IS_SOLID: u16 = 1 << 6;
const IS_FULL_CUBE: u16 = 1 << 7;
const IS_SOLID_BLOCK: u16 = 1 << 8;
const HAS_RANDOM_TICKS: u16 = 1 << 9;

// side_flags
const DOWN_SIDE_SOLID: u8 = 1 << 0;
const UP_SIDE_SOLID: u8 = 1 << 1;
const NORTH_SIDE_SOLID: u8 = 1 << 2;
const SOUTH_SIDE_SOLID: u8 = 1 << 3;
const WEST_SIDE_SOLID: u8 = 1 << 4;
const EAST_SIDE_SOLID: u8 = 1 << 5;
const DOWN_CENTER_SOLID: u8 = 1 << 6;
const UP_CENTER_SOLID: u8 = 1 << 7;
