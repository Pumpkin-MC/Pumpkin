pub mod interactive;
pub mod registry;
pub mod state;

use num_derive::FromPrimitive;
use pumpkin_data::block::CardinalDirection;
use pumpkin_util::math::vector3::Vector3;

pub use state::BlockState;

#[derive(FromPrimitive, PartialEq, Clone, Copy)]
pub enum BlockDirection {
    Down = 0,
    Up,
    North,
    South,
    West,
    East,
}

pub struct InvalidBlockFace;

impl TryFrom<i32> for BlockDirection {
    type Error = InvalidBlockFace;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Down),
            1 => Ok(Self::Up),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(InvalidBlockFace),
        }
    }
}

impl BlockDirection {
    pub fn to_offset(&self) -> Vector3<i32> {
        match self {
            BlockDirection::Down => (0, -1, 0),
            BlockDirection::Up => (0, 1, 0),
            BlockDirection::North => (0, 0, -1),
            BlockDirection::South => (0, 0, 1),
            BlockDirection::West => (-1, 0, 0),
            BlockDirection::East => (1, 0, 0),
        }
        .into()
    }
    pub fn opposite(&self) -> BlockDirection {
        match self {
            BlockDirection::Down => BlockDirection::Up,
            BlockDirection::Up => BlockDirection::Down,
            BlockDirection::North => BlockDirection::South,
            BlockDirection::South => BlockDirection::North,
            BlockDirection::West => BlockDirection::East,
            BlockDirection::East => BlockDirection::West,
        }
    }

    pub fn all() -> [BlockDirection; 6] {
        [
            BlockDirection::Down,
            BlockDirection::Up,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ]
    }
    pub fn update_order() -> [BlockDirection; 6] {
        [
            BlockDirection::West,
            BlockDirection::East,
            BlockDirection::Down,
            BlockDirection::Up,
            BlockDirection::North,
            BlockDirection::South,
        ]
    }

    pub fn horizontal() -> [BlockDirection; 4] {
        [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ]
    }

    pub fn vertical() -> [BlockDirection; 2] {
        [BlockDirection::Down, BlockDirection::Up]
    }

    pub fn to_cardinal_direction(&self) -> CardinalDirection {
        match self {
            BlockDirection::North => CardinalDirection::North,
            BlockDirection::South => CardinalDirection::South,
            BlockDirection::West => CardinalDirection::West,
            BlockDirection::East => CardinalDirection::East,
            _ => CardinalDirection::North,
        }
    }
}
