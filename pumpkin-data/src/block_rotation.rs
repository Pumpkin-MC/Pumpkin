//! Block rotation and mirroring transformations.
//!
//! These transformations are used to rotate and mirror blocks
//! when placing them in the world, matching vanilla Minecraft behavior.

use pumpkin_util::math::vector3::Vector3;
use serde::Deserialize;

/// Rotation around the Y axis in 90-degree increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rotation {
    /// No rotation (0 degrees)
    #[default]
    None,
    /// 90 degrees clockwise
    Clockwise90,
    /// 180 degrees
    Rotate180,
    /// 270 degrees clockwise (90 degrees counter-clockwise)
    CounterClockwise90,
}

impl Rotation {
    /// Returns all possible rotations.
    #[must_use]
    pub const fn values() -> [Self; 4] {
        [
            Self::None,
            Self::Clockwise90,
            Self::Rotate180,
            Self::CounterClockwise90,
        ]
    }

    /// Gets a random rotation from the given random value (0-3).
    #[must_use]
    pub const fn from_index(index: u8) -> Self {
        match index % 4 {
            0 => Self::None,
            1 => Self::Clockwise90,
            2 => Self::Rotate180,
            _ => Self::CounterClockwise90,
        }
    }

    /// Transforms a position within the template bounds according to this rotation.
    ///
    /// The position is rotated around the Y axis. The `size` parameter defines
    /// the template dimensions, used to calculate the pivot point.
    #[must_use]
    pub const fn transform_pos(&self, pos: Vector3<i32>, size: Vector3<i32>) -> Vector3<i32> {
        match self {
            Self::None => pos,
            Self::Clockwise90 => Vector3::new(size.z - 1 - pos.z, pos.y, pos.x),
            Self::Rotate180 => Vector3::new(size.x - 1 - pos.x, pos.y, size.z - 1 - pos.z),
            Self::CounterClockwise90 => Vector3::new(pos.z, pos.y, size.x - 1 - pos.x),
        }
    }

    /// Rotates an X/Z offset around the origin.
    ///
    /// Unlike `transform_pos` which rotates within template bounds,
    /// this rotates a simple offset (e.g. sub-template positioning).
    #[must_use]
    pub const fn rotate_offset(self, x: i32, z: i32) -> (i32, i32) {
        match self {
            Self::None => (x, z),
            Self::Clockwise90 => (-z, x),
            Self::Rotate180 => (-x, -z),
            Self::CounterClockwise90 => (z, -x),
        }
    }

    /// Rotates the template size dimensions according to this rotation.
    ///
    /// For 90 and 270 degree rotations, X and Z dimensions are swapped.
    #[must_use]
    pub const fn transform_size(&self, size: Vector3<i32>) -> Vector3<i32> {
        match self {
            Self::None | Self::Rotate180 => size,
            Self::Clockwise90 | Self::CounterClockwise90 => Vector3::new(size.z, size.y, size.x),
        }
    }

    /// Rotates a horizontal facing direction.
    ///
    /// Takes a facing string (north/south/east/west) and returns the rotated facing.
    #[must_use]
    pub fn rotate_facing(&self, facing: &str) -> &'static str {
        match self {
            Self::None => match facing {
                "north" => "north",
                "south" => "south",
                "east" => "east",
                "west" => "west",
                _ => leak_str(facing),
            },
            Self::Clockwise90 => match facing {
                "north" => "east",
                "east" => "south",
                "south" => "west",
                "west" => "north",
                _ => leak_str(facing),
            },
            Self::Rotate180 => match facing {
                "north" => "south",
                "south" => "north",
                "east" => "west",
                "west" => "east",
                _ => leak_str(facing),
            },
            Self::CounterClockwise90 => match facing {
                "north" => "west",
                "west" => "south",
                "south" => "east",
                "east" => "north",
                _ => leak_str(facing),
            },
        }
    }

    /// Rotates a horizontal axis.
    ///
    /// Takes an axis string (x/z) and returns the rotated axis.
    #[must_use]
    pub fn rotate_axis(&self, axis: &str) -> &'static str {
        match self {
            Self::None | Self::Rotate180 => match axis {
                "x" => "x",
                "z" => "z",
                _ => leak_str(axis),
            },
            Self::Clockwise90 | Self::CounterClockwise90 => match axis {
                "x" => "z",
                "z" => "x",
                _ => leak_str(axis),
            },
        }
    }

    /// Rotates a block rotation value (0-15, used for signs and banners).
    #[must_use]
    pub const fn rotate_block_rotation(&self, rotation: i32) -> i32 {
        match self {
            Self::None => rotation,
            Self::Clockwise90 => (rotation + 4) % 16,
            Self::Rotate180 => (rotation + 8) % 16,
            Self::CounterClockwise90 => (rotation + 12) % 16,
        }
    }

    /// Combines this rotation with another rotation.
    #[must_use]
    pub const fn then(&self, other: Self) -> Self {
        match self {
            Self::None => other,
            Self::Clockwise90 => match other {
                Self::None => Self::Clockwise90,
                Self::Clockwise90 => Self::Rotate180,
                Self::Rotate180 => Self::CounterClockwise90,
                Self::CounterClockwise90 => Self::None,
            },
            Self::Rotate180 => match other {
                Self::None => Self::Rotate180,
                Self::Clockwise90 => Self::CounterClockwise90,
                Self::Rotate180 => Self::None,
                Self::CounterClockwise90 => Self::Clockwise90,
            },
            Self::CounterClockwise90 => match other {
                Self::None => Self::CounterClockwise90,
                Self::Clockwise90 => Self::None,
                Self::Rotate180 => Self::Clockwise90,
                Self::CounterClockwise90 => Self::Rotate180,
            },
        }
    }

    /// Returns the inverse of this rotation.
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Clockwise90 => Self::CounterClockwise90,
            Self::Rotate180 => Self::Rotate180,
            Self::CounterClockwise90 => Self::Clockwise90,
        }
    }

    /// Converts rotation to a primary axis for bounding box creation.
    #[must_use]
    pub const fn to_axis(self) -> pumpkin_util::math::vector3::Axis {
        match self {
            Self::None | Self::Rotate180 => pumpkin_util::math::vector3::Axis::Z,
            Self::Clockwise90 | Self::CounterClockwise90 => pumpkin_util::math::vector3::Axis::X,
        }
    }
}

/// Mirror transformation for structure templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mirror {
    /// No mirroring
    #[default]
    None,
    /// Mirror along the X axis (left-right flip when looking north)
    LeftRight,
    /// Mirror along the Z axis (front-back flip)
    FrontBack,
}

impl Mirror {
    /// Returns all possible mirrors.
    #[must_use]
    pub const fn values() -> [Self; 3] {
        [Self::None, Self::LeftRight, Self::FrontBack]
    }

    /// Transforms a position within the template bounds according to this mirror.
    #[must_use]
    pub const fn transform_pos(&self, pos: Vector3<i32>, size: Vector3<i32>) -> Vector3<i32> {
        match self {
            Self::None => pos,
            Self::LeftRight => Vector3::new(size.x - 1 - pos.x, pos.y, pos.z),
            Self::FrontBack => Vector3::new(pos.x, pos.y, size.z - 1 - pos.z),
        }
    }

    /// Mirrors a horizontal facing direction.
    #[must_use]
    pub fn mirror_facing(&self, facing: &str) -> &'static str {
        match self {
            Self::None => match facing {
                "north" => "north",
                "south" => "south",
                "east" => "east",
                "west" => "west",
                _ => leak_str(facing),
            },
            Self::LeftRight => match facing {
                "east" => "west",
                "west" => "east",
                "north" => "north",
                "south" => "south",
                _ => leak_str(facing),
            },
            Self::FrontBack => match facing {
                "north" => "south",
                "south" => "north",
                "east" => "east",
                "west" => "west",
                _ => leak_str(facing),
            },
        }
    }

    /// Mirrors a block rotation value (0-15, used for signs and banners).
    #[must_use]
    pub const fn mirror_block_rotation(&self, rotation: i32) -> i32 {
        match self {
            Self::None => rotation,
            Self::LeftRight => (16 - rotation) % 16,
            Self::FrontBack => (8 - rotation + 16) % 16,
        }
    }

    /// Returns the rotation needed to achieve this mirror from a base rotation.
    #[must_use]
    pub const fn get_rotation(&self, rotation: Rotation) -> Rotation {
        match self {
            Self::None => rotation,
            Self::LeftRight => match rotation {
                Rotation::None => Rotation::None,
                Rotation::Clockwise90 => Rotation::CounterClockwise90,
                Rotation::Rotate180 => Rotation::Rotate180,
                Rotation::CounterClockwise90 => Rotation::Clockwise90,
            },
            Self::FrontBack => match rotation {
                Rotation::None => Rotation::Rotate180,
                Rotation::Clockwise90 => Rotation::Clockwise90,
                Rotation::Rotate180 => Rotation::None,
                Rotation::CounterClockwise90 => Rotation::CounterClockwise90,
            },
        }
    }
}

/// Leaks a string to get a 'static str.
/// This is used for non-standard property values that aren't covered by static strings.
pub fn leak_str(s: &str) -> &'static str {
    s.to_string().leak()
}
