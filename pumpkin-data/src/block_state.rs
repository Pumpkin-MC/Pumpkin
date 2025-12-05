use pumpkin_util::math::vector3::Vector3;

use crate::block_properties::{COLLISION_SHAPES, Instrument};
use crate::{Block, BlockDirection, CollisionShape};

#[derive(Debug)]
pub struct BlockState {
    pub id: u16,
    pub state_flags: u16,
    pub side_flags: u8,
    pub instrument: Instrument,
    pub luminance: u8,
    pub piston_behavior: PistonBehavior,
    pub hardness: f32,
    pub collision_shapes: &'static [u16],
    pub outline_shapes: &'static [u16],
    pub opacity: u8,
    /// u16::MAX is used as None
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

// Add your methods here
impl BlockState {
    pub const fn is_air(&self) -> bool {
        self.state_flags & IS_AIR != 0
    }

    pub const fn burnable(&self) -> bool {
        self.state_flags & BURNABLE != 0
    }

    pub const fn tool_required(&self) -> bool {
        self.state_flags & TOOL_REQUIRED != 0
    }

    pub const fn sided_transparency(&self) -> bool {
        self.state_flags & SIDED_TRANSPARENCY != 0
    }

    pub const fn replaceable(&self) -> bool {
        self.state_flags & REPLACEABLE != 0
    }

    pub const fn is_liquid(&self) -> bool {
        self.state_flags & IS_LIQUID != 0
    }

    /// Returns the legacy value for whether a block is solid.
    pub const fn is_solid(&self) -> bool {
        self.state_flags & IS_SOLID != 0
    }

    pub const fn is_full_cube(&self) -> bool {
        self.state_flags & IS_FULL_CUBE != 0
    }

    /// Returns whether the block is solid.
    /// Solid blocks conduct redstone and block redstone wire.
    /// Non-solid blocks don't allow redstone wire on top to propagate their signal downwards in java.
    pub const fn is_solid_block(&self) -> bool {
        self.state_flags & IS_SOLID_BLOCK != 0
    }

    pub const fn has_random_ticks(&self) -> bool {
        self.state_flags & HAS_RANDOM_TICKS != 0
    }

    ///isSideSolidFullSquare() in Java!
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
    pub const fn is_center_solid(&self, side: BlockDirection) -> bool {
        match side {
            BlockDirection::Down => self.side_flags & DOWN_CENTER_SOLID != 0,
            BlockDirection::Up => self.side_flags & UP_CENTER_SOLID != 0,
            _ => unreachable!(),
        }
    }

    pub fn get_block_collision_shapes(&self) -> Vec<CollisionShape> {
        self.collision_shapes
            .iter()
            .map(|&id| COLLISION_SHAPES[id as usize])
            .collect()
    }

    pub fn get_block_outline_shapes(&self) -> Option<Vec<CollisionShape>> {
        let mut shapes: Vec<CollisionShape> = self
            .outline_shapes
            .iter()
            .map(|&id| COLLISION_SHAPES[id as usize])
            .collect();

        let block = Block::from_state_id(self.id);
        if let Some(props) = block.properties(self.id) {
            let is_waterlogged = props
                .to_props()
                .iter()
                .any(|(k, v)| *k == "waterlogged" && *v == "true");

            if is_waterlogged {
                shapes.push(CollisionShape::new(
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.875, 1.0),
                ));
            }
        }

        Some(shapes)
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
