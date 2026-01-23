use crate::block_properties::{Axis, Facing, HorizontalAxis, HorizontalFacing};
use pumpkin_util::{
    math::vector3::{Axis as MathAxis, Vector3},
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

impl From<MathAxis> for Axis {
    fn from(a: MathAxis) -> Self {
        match a {
            MathAxis::X => Self::X,
            MathAxis::Y => Self::Y,
            MathAxis::Z => Self::Z,
        }
    }
}
impl From<Axis> for MathAxis {
    fn from(a: Axis) -> Self {
        match a {
            Axis::X => Self::X,
            Axis::Y => Self::Y,
            Axis::Z => Self::Z,
        }
    }
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
    #[must_use]
    pub const fn to_index(&self) -> u8 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
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

    pub fn random_horizontal(random: &mut RandomGenerator) -> Self {
        Self::horizontal()[random.next_bounded_i32(Self::horizontal().len() as i32 - 1) as usize]
    }

    #[must_use]
    pub fn by_index(index: usize) -> Option<Self> {
        Self::all().get(index % Self::all().len()).copied()
    }

    #[must_use]
    pub fn to_offset(&self) -> Vector3<i32> {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
        .into()
    }

    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    #[must_use]
    pub const fn positive(&self) -> bool {
        matches!(self, Self::South | Self::East | Self::Up)
    }

    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Down,
            Self::Up,
            Self::North,
            Self::South,
            Self::West,
            Self::East,
        ]
    }
    #[must_use]
    pub const fn update_order() -> [Self; 6] {
        [
            Self::West,
            Self::East,
            Self::Down,
            Self::Up,
            Self::North,
            Self::South,
        ]
    }

    #[must_use]
    pub const fn abstract_block_update_order() -> [Self; 6] {
        [
            Self::West,
            Self::East,
            Self::North,
            Self::South,
            Self::Down,
            Self::Up,
        ]
    }

    #[must_use]
    pub const fn horizontal() -> [Self; 4] {
        [Self::North, Self::South, Self::West, Self::East]
    }

    #[must_use]
    pub const fn flow_directions() -> [Self; 5] {
        [Self::Down, Self::North, Self::South, Self::West, Self::East]
    }

    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::North | Self::South | Self::West | Self::East)
    }

    #[must_use]
    pub const fn vertical() -> [Self; 2] {
        [Self::Down, Self::Up]
    }

    #[must_use]
    pub const fn to_horizontal_facing(&self) -> Option<HorizontalFacing> {
        match self {
            Self::North => Some(HorizontalFacing::North),
            Self::South => Some(HorizontalFacing::South),
            Self::West => Some(HorizontalFacing::West),
            Self::East => Some(HorizontalFacing::East),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_horizontal_axis(&self) -> Option<HorizontalAxis> {
        match self {
            Self::North | Self::South => Some(HorizontalAxis::Z),
            Self::West | Self::East => Some(HorizontalAxis::X),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_cardinal_direction(&self) -> HorizontalFacing {
        match self {
            Self::North => HorizontalFacing::North,
            Self::South => HorizontalFacing::South,
            Self::West => HorizontalFacing::West,
            Self::East => HorizontalFacing::East,
            _ => HorizontalFacing::North,
        }
    }

    #[must_use]
    pub const fn from_cardinal_direction(direction: HorizontalFacing) -> Self {
        match direction {
            HorizontalFacing::North => Self::North,
            HorizontalFacing::South => Self::South,
            HorizontalFacing::West => Self::West,
            HorizontalFacing::East => Self::East,
        }
    }
    #[must_use]
    pub const fn to_axis(&self) -> Axis {
        match self {
            Self::North | Self::South => Axis::Z,
            Self::West | Self::East => Axis::X,
            Self::Up | Self::Down => Axis::Y,
        }
    }

    #[must_use]
    pub const fn to_facing(&self) -> Facing {
        match self {
            Self::North => Facing::North,
            Self::South => Facing::South,
            Self::West => Facing::West,
            Self::East => Facing::East,
            Self::Up => Facing::Up,
            Self::Down => Facing::Down,
        }
    }

    #[must_use]
    pub const fn rotate_clockwise(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::Up => Self::East,
            Self::Down => Self::West,
        }
    }

    #[must_use]
    pub const fn rotate_counter_clockwise(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            Self::Up => Self::West,
            Self::Down => Self::East,
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
