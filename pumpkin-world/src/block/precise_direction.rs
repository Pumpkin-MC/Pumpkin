use num_derive::FromPrimitive;
use pumpkin_data::block::{HorizontalFacing, Integer0To15};

#[derive(FromPrimitive, PartialEq, Clone, Copy, Debug)]
pub enum PreciseDirection {
    South = 0,
    SouthSouthWest,
    SouthWest,
    WestSouthWest,
    West,
    WestNorthWest,
    NorthWest,
    NorthNorthWest,
    North,
    NorthNorthEast,
    NorthEast,
    EastNorthEast,
    East,
    EastSouthEast,
    SouthEast,
    SouthSouthEast,
}

pub struct InvalidInputRange;

impl From<f32> for PreciseDirection {
    fn from(value: f32) -> PreciseDirection {
        match value {
            0.0..=22.5 => PreciseDirection::South,
            22.5..=45.0 => PreciseDirection::SouthSouthWest,
            45.0..=67.5 => PreciseDirection::SouthWest,
            67.5..=90.0 => PreciseDirection::WestSouthWest,
            90.0..=112.5 => PreciseDirection::West,
            112.5..=135.0 => PreciseDirection::WestNorthWest,
            135.0..=157.5 => PreciseDirection::NorthWest,
            157.5..=180.0 => PreciseDirection::NorthNorthWest,
            180.0..=202.5 => PreciseDirection::North,
            202.5..=225.0 => PreciseDirection::NorthNorthEast,
            225.0..=247.5 => PreciseDirection::NorthEast,
            247.5..=270.0 => PreciseDirection::EastNorthEast,
            270.0..=292.5 => PreciseDirection::East,
            292.5..=315.0 => PreciseDirection::EastSouthEast,
            315.0..=337.5 => PreciseDirection::SouthEast,
            337.5..=360.0 => PreciseDirection::SouthSouthEast,
            _ => PreciseDirection::South, // Default case, should not occur
        }
    }
}

impl PreciseDirection {
    pub fn to_horizontal_direction(&self) -> HorizontalFacing {
        // This is what this is from
        // 0.0..=45.0 | 315.0..=360.0 => HorizontalFacing::South,
        // 45.0..=135.0 => HorizontalFacing::West,
        // 135.0..=225.0 => HorizontalFacing::North,
        // 225.0..=315.0 => HorizontalFacing::East,

        match self {
            // 0.0..=45.0 | 315.0..=360.0 => South
            PreciseDirection::South => HorizontalFacing::South,
            PreciseDirection::SouthSouthWest => HorizontalFacing::South,
            PreciseDirection::SouthWest => HorizontalFacing::South,
            PreciseDirection::SouthSouthEast => HorizontalFacing::South,
            PreciseDirection::SouthEast => HorizontalFacing::South,

            // 45.0..=135.0 => West
            PreciseDirection::WestSouthWest => HorizontalFacing::West,
            PreciseDirection::West => HorizontalFacing::West,
            PreciseDirection::WestNorthWest => HorizontalFacing::West,

            // 135.0..=225.0 => North
            PreciseDirection::North => HorizontalFacing::North,
            PreciseDirection::NorthNorthEast => HorizontalFacing::North,
            PreciseDirection::NorthNorthWest => HorizontalFacing::North,
            PreciseDirection::NorthEast => HorizontalFacing::North,
            PreciseDirection::NorthWest => HorizontalFacing::North,

            // 225.0..=315.0 => East
            PreciseDirection::EastNorthEast => HorizontalFacing::East,
            PreciseDirection::East => HorizontalFacing::East,
            PreciseDirection::EastSouthEast => HorizontalFacing::East,
        }
    }

    pub fn to_integer_0_to_15(&self) -> Integer0To15 {
        match self {
            // Taken from wiki
            PreciseDirection::South => Integer0To15::L0,
            PreciseDirection::SouthSouthWest => Integer0To15::L1,
            PreciseDirection::SouthWest => Integer0To15::L2,
            PreciseDirection::WestSouthWest => Integer0To15::L3,
            PreciseDirection::West => Integer0To15::L4,
            PreciseDirection::WestNorthWest => Integer0To15::L5,
            PreciseDirection::NorthWest => Integer0To15::L6,
            PreciseDirection::NorthNorthWest => Integer0To15::L7,
            PreciseDirection::North => Integer0To15::L8,
            PreciseDirection::NorthNorthEast => Integer0To15::L9,
            PreciseDirection::NorthEast => Integer0To15::L10,
            PreciseDirection::EastNorthEast => Integer0To15::L11,
            PreciseDirection::East => Integer0To15::L12,
            PreciseDirection::EastSouthEast => Integer0To15::L13,
            PreciseDirection::SouthEast => Integer0To15::L14,
            PreciseDirection::SouthSouthEast => Integer0To15::L15,
        }
    }

    pub fn opposite(&self) -> PreciseDirection {
        match self {
            // Opposite of South
            PreciseDirection::South => PreciseDirection::North,
            PreciseDirection::SouthSouthWest => PreciseDirection::NorthNorthEast,
            PreciseDirection::SouthWest => PreciseDirection::NorthEast,
            PreciseDirection::SouthSouthEast => PreciseDirection::NorthNorthWest,
            PreciseDirection::SouthEast => PreciseDirection::NorthWest,

            // Opposite of West
            PreciseDirection::WestSouthWest => PreciseDirection::EastNorthEast,
            PreciseDirection::West => PreciseDirection::East,
            PreciseDirection::WestNorthWest => PreciseDirection::EastSouthEast,

            // Opposite of North
            PreciseDirection::North => PreciseDirection::South,
            PreciseDirection::NorthNorthEast => PreciseDirection::SouthSouthWest,
            PreciseDirection::NorthEast => PreciseDirection::SouthWest,
            PreciseDirection::NorthNorthWest => PreciseDirection::SouthSouthEast,
            PreciseDirection::NorthWest => PreciseDirection::SouthEast,

            // Opposite of East
            PreciseDirection::EastNorthEast => PreciseDirection::WestSouthWest,
            PreciseDirection::East => PreciseDirection::West,
            PreciseDirection::EastSouthEast => PreciseDirection::WestNorthWest,
        }
    }
}
