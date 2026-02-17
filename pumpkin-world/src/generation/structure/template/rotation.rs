//! Block rotation and mirroring transformations for structure templates.
//!
//! These transformations are used to rotate and mirror structure templates
//! when placing them in the world, matching vanilla Minecraft behavior.

use pumpkin_util::math::vector3::Vector3;

/// Rotation around the Y axis in 90-degree increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockRotation {
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

impl BlockRotation {
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
    pub fn transform_pos(&self, pos: Vector3<i32>, size: Vector3<i32>) -> Vector3<i32> {
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
    pub fn rotate_offset(self, x: i32, z: i32) -> (i32, i32) {
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
        let a = *self as u8;
        let b = other as u8;
        Self::from_index((a + b) % 4)
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
}

/// Mirror transformation for structure templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockMirror {
    /// No mirroring
    #[default]
    None,
    /// Mirror along the X axis (left-right flip when looking north)
    LeftRight,
    /// Mirror along the Z axis (front-back flip)
    FrontBack,
}

impl BlockMirror {
    /// Returns all possible mirrors.
    #[must_use]
    pub const fn values() -> [Self; 3] {
        [Self::None, Self::LeftRight, Self::FrontBack]
    }

    /// Transforms a position within the template bounds according to this mirror.
    #[must_use]
    pub fn transform_pos(&self, pos: Vector3<i32>, size: Vector3<i32>) -> Vector3<i32> {
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
    pub const fn get_rotation(&self, rotation: BlockRotation) -> BlockRotation {
        match self {
            Self::None => rotation,
            Self::LeftRight => match rotation {
                BlockRotation::None => BlockRotation::None,
                BlockRotation::Clockwise90 => BlockRotation::CounterClockwise90,
                BlockRotation::Rotate180 => BlockRotation::Rotate180,
                BlockRotation::CounterClockwise90 => BlockRotation::Clockwise90,
            },
            Self::FrontBack => match rotation {
                BlockRotation::None => BlockRotation::Rotate180,
                BlockRotation::Clockwise90 => BlockRotation::Clockwise90,
                BlockRotation::Rotate180 => BlockRotation::None,
                BlockRotation::CounterClockwise90 => BlockRotation::CounterClockwise90,
            },
        }
    }
}

/// Leaks a string to get a 'static str.
/// This is used for non-standard property values that aren't covered by static strings.
fn leak_str(s: &str) -> &'static str {
    s.to_string().leak()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_transform_pos() {
        let size = Vector3::new(3, 5, 4); // 3 wide, 5 tall, 4 deep
        let pos = Vector3::new(0, 2, 0);

        // No rotation: position unchanged
        assert_eq!(BlockRotation::None.transform_pos(pos, size), pos);

        // Clockwise 90: (x, y, z) -> (size.z - 1 - z, y, x)
        assert_eq!(
            BlockRotation::Clockwise90.transform_pos(pos, size),
            Vector3::new(3, 2, 0)
        );

        // 180: (x, y, z) -> (size.x - 1 - x, y, size.z - 1 - z)
        assert_eq!(
            BlockRotation::Rotate180.transform_pos(pos, size),
            Vector3::new(2, 2, 3)
        );

        // Counter-clockwise 90: (x, y, z) -> (z, y, size.x - 1 - x)
        assert_eq!(
            BlockRotation::CounterClockwise90.transform_pos(pos, size),
            Vector3::new(0, 2, 2)
        );
    }

    #[test]
    fn test_rotation_facing() {
        assert_eq!(BlockRotation::Clockwise90.rotate_facing("north"), "east");
        assert_eq!(BlockRotation::Clockwise90.rotate_facing("east"), "south");
        assert_eq!(BlockRotation::Rotate180.rotate_facing("north"), "south");
        assert_eq!(
            BlockRotation::CounterClockwise90.rotate_facing("north"),
            "west"
        );
    }

    #[test]
    fn test_mirror_facing() {
        assert_eq!(BlockMirror::LeftRight.mirror_facing("east"), "west");
        assert_eq!(BlockMirror::LeftRight.mirror_facing("north"), "north");
        assert_eq!(BlockMirror::FrontBack.mirror_facing("north"), "south");
        assert_eq!(BlockMirror::FrontBack.mirror_facing("east"), "east");
    }

    #[test]
    fn test_rotation_inverse() {
        for rotation in BlockRotation::values() {
            let inverse = rotation.inverse();
            let combined = rotation.then(inverse);
            assert_eq!(combined, BlockRotation::None);
        }
    }

    #[test]
    fn test_transform_size() {
        let size = Vector3::new(3, 5, 7);

        assert_eq!(BlockRotation::None.transform_size(size), size);
        assert_eq!(
            BlockRotation::Clockwise90.transform_size(size),
            Vector3::new(7, 5, 3)
        );
        assert_eq!(BlockRotation::Rotate180.transform_size(size), size);
        assert_eq!(
            BlockRotation::CounterClockwise90.transform_size(size),
            Vector3::new(7, 5, 3)
        );
    }
}
