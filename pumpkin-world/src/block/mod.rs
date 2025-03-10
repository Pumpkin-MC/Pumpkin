pub mod interactive;
pub mod registry;
pub mod state;

use num_derive::FromPrimitive;
use pumpkin_data::block::{Axis, HorizontalFacing};
use pumpkin_util::math::vector3::Vector3;

use serde::Deserialize;
pub use state::ChunkBlockState;

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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockStateCodec {
    pub name: String,
    // TODO: properties...
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

    pub fn to_cardinal_direction(&self) -> HorizontalFacing {
        match self {
            BlockDirection::North => HorizontalFacing::North,
            BlockDirection::South => HorizontalFacing::South,
            BlockDirection::West => HorizontalFacing::West,
            BlockDirection::East => HorizontalFacing::East,
            _ => HorizontalFacing::North,
        }
    }

    pub fn from_cardinal_direction(direction: HorizontalFacing) -> BlockDirection {
        match direction {
            HorizontalFacing::North => BlockDirection::North,
            HorizontalFacing::South => BlockDirection::South,
            HorizontalFacing::West => BlockDirection::West,
            HorizontalFacing::East => BlockDirection::East,
        }
    }
    pub fn to_axis(&self) -> Axis {
        match self {
            BlockDirection::North | BlockDirection::South => Axis::Z,
            BlockDirection::West | BlockDirection::East => Axis::X,
            BlockDirection::Up | BlockDirection::Down => Axis::Y,
        }
    }

    pub fn rotate_clockwise(&self) -> BlockDirection {
        match self {
            BlockDirection::North => BlockDirection::East,
            BlockDirection::East => BlockDirection::South,
            BlockDirection::South => BlockDirection::West,
            BlockDirection::West => BlockDirection::North,
            BlockDirection::Up => BlockDirection::East,
            BlockDirection::Down => BlockDirection::West,
        }
    }
}
