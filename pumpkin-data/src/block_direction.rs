use crate::block_properties::{Axis, Facing, HorizontalFacing};
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

#[repr(u8)]
#[derive(PartialEq, Clone, Copy, Debug, Hash, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    pub fn to_index(&self) -> u8 {
        match self {
            BlockDirection::Down => 0,
            BlockDirection::Up => 1,
            BlockDirection::North => 2,
            BlockDirection::South => 3,
            BlockDirection::West => 4,
            BlockDirection::East => 5,
        }
    }

    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Down),
            1 => Some(Self::Up),
            2 => Some(Self::North),
            3 => Some(Self::South),
            4 => Some(Self::West),
            5 => Some(Self::East),
            _ => None,
        }
    }

    pub fn random(random: &mut RandomGenerator) -> Self {
        Self::all()[random.next_bounded_i32(Self::all().len() as i32 - 1) as usize]
    }

    pub fn by_index(index: usize) -> Option<Self> {
        Self::all().get(index % Self::all().len()).cloned()
    }

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

    pub fn abstract_block_update_order() -> [BlockDirection; 6] {
        [
            BlockDirection::West,
            BlockDirection::East,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::Down,
            BlockDirection::Up,
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

    pub fn is_horizontal(&self) -> bool {
        matches!(
            self,
            BlockDirection::North
                | BlockDirection::South
                | BlockDirection::West
                | BlockDirection::East
        )
    }

    pub fn vertical() -> [BlockDirection; 2] {
        [BlockDirection::Down, BlockDirection::Up]
    }

    pub fn to_horizontal_facing(&self) -> Option<HorizontalFacing> {
        match self {
            BlockDirection::North => Some(HorizontalFacing::North),
            BlockDirection::South => Some(HorizontalFacing::South),
            BlockDirection::West => Some(HorizontalFacing::West),
            BlockDirection::East => Some(HorizontalFacing::East),
            _ => None,
        }
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

    pub fn to_facing(&self) -> Facing {
        match self {
            BlockDirection::North => Facing::North,
            BlockDirection::South => Facing::South,
            BlockDirection::West => Facing::West,
            BlockDirection::East => Facing::East,
            BlockDirection::Up => Facing::Up,
            BlockDirection::Down => Facing::Down,
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

pub trait FacingExt {
    fn to_block_direction(&self) -> BlockDirection;
}

impl FacingExt for Facing {
    fn to_block_direction(&self) -> BlockDirection {
        match self {
            Self::North => BlockDirection::North,
            Self::South => BlockDirection::South,
            Self::West => BlockDirection::West,
            Self::East => BlockDirection::East,
            Self::Up => BlockDirection::Up,
            Self::Down => BlockDirection::Down,
        }
    }
}

pub trait HorizontalFacingExt {
    fn to_block_direction(&self) -> BlockDirection;
}

impl HorizontalFacingExt for HorizontalFacing {
    fn to_block_direction(&self) -> BlockDirection {
        match self {
            Self::North => BlockDirection::North,
            Self::South => BlockDirection::South,
            Self::West => BlockDirection::West,
            Self::East => BlockDirection::East,
        }
    }
}
