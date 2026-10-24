//! Block rotation and mirroring transformations.
//!
//! These transformations are used to rotate and mirror blocks
//! when placing them in the world, matching vanilla Minecraft behavior.

use crate::BlockDirection;
use crate::block_properties::{
    Axis, DoorHinge, EastRedstone, EastWall, EnumVariants, Facing, HorizontalFacing, NorthRedstone,
    NorthWall, Orientation, Property, RailShape, SouthRedstone, SouthWall, StairsShape,
    WestRedstone, WestWall,
};
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

    /// Rotates a facing direction around the Y axis.
    #[must_use]
    pub const fn rotate_facing(&self, facing: Facing) -> Facing {
        match self {
            Self::None => facing,
            Self::Clockwise90 => match facing {
                Facing::North => Facing::East,
                Facing::East => Facing::South,
                Facing::South => Facing::West,
                Facing::West => Facing::North,
                Facing::Up | Facing::Down => facing,
            },
            Self::Rotate180 => match facing {
                Facing::North => Facing::South,
                Facing::South => Facing::North,
                Facing::East => Facing::West,
                Facing::West => Facing::East,
                Facing::Up | Facing::Down => facing,
            },
            Self::CounterClockwise90 => match facing {
                Facing::North => Facing::West,
                Facing::West => Facing::South,
                Facing::South => Facing::East,
                Facing::East => Facing::North,
                Facing::Up | Facing::Down => facing,
            },
        }
    }

    /// Rotates an axis around the Y axis.
    #[must_use]
    pub const fn rotate_axis(&self, axis: Axis) -> Axis {
        match self {
            Self::None | Self::Rotate180 => axis,
            Self::Clockwise90 | Self::CounterClockwise90 => match axis {
                Axis::X => Axis::Z,
                Axis::Z => Axis::X,
                Axis::Y => Axis::Y,
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

    /// Rotates a 3D block direction according to this rotation around the Y axis.
    #[must_use]
    pub const fn rotate(&self, direction: BlockDirection) -> BlockDirection {
        if matches!(direction, BlockDirection::Down | BlockDirection::Up) {
            return direction;
        }
        match self {
            Self::None => direction,
            Self::Clockwise90 => direction.rotate_clockwise(),
            Self::Rotate180 => direction.opposite(),
            Self::CounterClockwise90 => direction.rotate_counter_clockwise(),
        }
    }

    /// Rotates a horizontal facing direction according to this rotation.
    #[must_use]
    pub const fn rotate_horizontal(&self, facing: HorizontalFacing) -> HorizontalFacing {
        match self {
            Self::None => facing,
            Self::Clockwise90 => facing.rotate_clockwise(),
            Self::Rotate180 => facing.opposite(),
            Self::CounterClockwise90 => facing.rotate_counter_clockwise(),
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
    /// Mirror along the Z axis (inverting Z: north <-> south)
    LeftRight,
    /// Mirror along the X axis (inverting X: east <-> west)
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
            Self::LeftRight => Vector3::new(pos.x, pos.y, size.z - 1 - pos.z),
            Self::FrontBack => Vector3::new(size.x - 1 - pos.x, pos.y, pos.z),
        }
    }

    /// Mirrors a facing direction across this mirror plane.
    #[must_use]
    pub const fn mirror_facing(&self, facing: Facing) -> Facing {
        match self {
            Self::None => facing,
            Self::LeftRight => match facing {
                Facing::North => Facing::South,
                Facing::South => Facing::North,
                Facing::East | Facing::West | Facing::Up | Facing::Down => facing,
            },
            Self::FrontBack => match facing {
                Facing::East => Facing::West,
                Facing::West => Facing::East,
                Facing::North | Facing::South | Facing::Up | Facing::Down => facing,
            },
        }
    }

    /// Mirrors a block rotation value (0-15, used for signs and banners).
    #[must_use]
    pub const fn mirror_block_rotation(&self, rotation: i32) -> i32 {
        match self {
            Self::None => rotation,
            Self::LeftRight => (8 - rotation + 16) % 16,
            Self::FrontBack => (16 - rotation) % 16,
        }
    }

    /// Returns the rotation needed to achieve this mirror from a base rotation.
    #[must_use]
    pub const fn get_rotation(&self, rotation: Rotation) -> Rotation {
        match self {
            Self::None => rotation,
            Self::LeftRight => match rotation {
                Rotation::None => Rotation::Rotate180,
                Rotation::Clockwise90 => Rotation::Clockwise90,
                Rotation::Rotate180 => Rotation::None,
                Rotation::CounterClockwise90 => Rotation::CounterClockwise90,
            },
            Self::FrontBack => match rotation {
                Rotation::None => Rotation::None,
                Rotation::Clockwise90 => Rotation::CounterClockwise90,
                Rotation::Rotate180 => Rotation::Rotate180,
                Rotation::CounterClockwise90 => Rotation::Clockwise90,
            },
        }
    }

    /// Mirrors a 3D block direction according to this mirror plane.
    #[must_use]
    pub const fn mirror(&self, direction: BlockDirection) -> BlockDirection {
        match self {
            Self::None => direction,
            Self::LeftRight => match direction {
                BlockDirection::North => BlockDirection::South,
                BlockDirection::South => BlockDirection::North,
                _ => direction,
            },
            Self::FrontBack => match direction {
                BlockDirection::East => BlockDirection::West,
                BlockDirection::West => BlockDirection::East,
                _ => direction,
            },
        }
    }

    /// Mirrors a horizontal facing direction according to this mirror plane.
    #[must_use]
    pub const fn mirror_horizontal(&self, facing: HorizontalFacing) -> HorizontalFacing {
        match self {
            Self::None => facing,
            Self::LeftRight => match facing {
                HorizontalFacing::North => HorizontalFacing::South,
                HorizontalFacing::South => HorizontalFacing::North,
                _ => facing,
            },
            Self::FrontBack => match facing {
                HorizontalFacing::East => HorizontalFacing::West,
                HorizontalFacing::West => HorizontalFacing::East,
                _ => facing,
            },
        }
    }

    /// Returns the rotation required for mirroring a direction with this mirror.
    #[must_use]
    pub const fn get_rotation_for_direction(&self, direction: BlockDirection) -> Rotation {
        match self {
            Self::LeftRight
                if matches!(direction, BlockDirection::North | BlockDirection::South) =>
            {
                Rotation::Rotate180
            }
            Self::FrontBack if matches!(direction, BlockDirection::East | BlockDirection::West) => {
                Rotation::Rotate180
            }
            _ => Rotation::None,
        }
    }

    /// Returns the rotation required for mirroring a horizontal facing with this mirror.
    #[must_use]
    pub const fn get_rotation_for_horizontal(&self, facing: HorizontalFacing) -> Rotation {
        match self {
            Self::LeftRight
                if matches!(facing, HorizontalFacing::North | HorizontalFacing::South) =>
            {
                Rotation::Rotate180
            }
            Self::FrontBack
                if matches!(facing, HorizontalFacing::East | HorizontalFacing::West) =>
            {
                Rotation::Rotate180
            }
            _ => Rotation::None,
        }
    }
}

/// Transforms rail shape based on rotation and mirror matching vanilla Minecraft BaseRailBlock.
#[must_use]
pub const fn transform_rail_shape(
    mut shape: RailShape,
    rotation: Rotation,
    mirror: Mirror,
) -> RailShape {
    match mirror {
        Mirror::LeftRight => {
            shape = match shape {
                RailShape::AscendingNorth => RailShape::AscendingSouth,
                RailShape::AscendingSouth => RailShape::AscendingNorth,
                RailShape::SouthEast => RailShape::NorthEast,
                RailShape::SouthWest => RailShape::NorthWest,
                RailShape::NorthWest => RailShape::SouthWest,
                RailShape::NorthEast => RailShape::SouthEast,
                _ => shape,
            };
        }
        Mirror::FrontBack => {
            shape = match shape {
                RailShape::AscendingEast => RailShape::AscendingWest,
                RailShape::AscendingWest => RailShape::AscendingEast,
                RailShape::SouthEast => RailShape::SouthWest,
                RailShape::SouthWest => RailShape::SouthEast,
                RailShape::NorthWest => RailShape::NorthEast,
                RailShape::NorthEast => RailShape::NorthWest,
                _ => shape,
            };
        }
        Mirror::None => {}
    }

    match rotation {
        Rotation::Rotate180 => match shape {
            RailShape::AscendingEast => RailShape::AscendingWest,
            RailShape::AscendingWest => RailShape::AscendingEast,
            RailShape::AscendingNorth => RailShape::AscendingSouth,
            RailShape::AscendingSouth => RailShape::AscendingNorth,
            RailShape::SouthEast => RailShape::NorthWest,
            RailShape::SouthWest => RailShape::NorthEast,
            RailShape::NorthWest => RailShape::SouthEast,
            RailShape::NorthEast => RailShape::SouthWest,
            _ => shape,
        },
        Rotation::CounterClockwise90 => match shape {
            RailShape::AscendingEast => RailShape::AscendingNorth,
            RailShape::AscendingWest => RailShape::AscendingSouth,
            RailShape::AscendingNorth => RailShape::AscendingWest,
            RailShape::AscendingSouth => RailShape::AscendingEast,
            RailShape::NorthSouth => RailShape::EastWest,
            RailShape::EastWest => RailShape::NorthSouth,
            RailShape::SouthEast => RailShape::NorthEast,
            RailShape::SouthWest => RailShape::SouthEast,
            RailShape::NorthWest => RailShape::SouthWest,
            RailShape::NorthEast => RailShape::NorthWest,
        },
        Rotation::Clockwise90 => match shape {
            RailShape::AscendingEast => RailShape::AscendingSouth,
            RailShape::AscendingWest => RailShape::AscendingNorth,
            RailShape::AscendingNorth => RailShape::AscendingEast,
            RailShape::AscendingSouth => RailShape::AscendingWest,
            RailShape::NorthSouth => RailShape::EastWest,
            RailShape::EastWest => RailShape::NorthSouth,
            RailShape::SouthEast => RailShape::SouthWest,
            RailShape::SouthWest => RailShape::NorthWest,
            RailShape::NorthWest => RailShape::NorthEast,
            RailShape::NorthEast => RailShape::SouthEast,
        },
        Rotation::None => shape,
    }
}

const fn transform_orientation(
    orientation: Orientation,
    rotation: Rotation,
    mirror: Mirror,
) -> Orientation {
    let (front, top) = match orientation {
        Orientation::DownEast => (Facing::Down, Facing::East),
        Orientation::DownNorth => (Facing::Down, Facing::North),
        Orientation::DownSouth => (Facing::Down, Facing::South),
        Orientation::DownWest => (Facing::Down, Facing::West),
        Orientation::UpEast => (Facing::Up, Facing::East),
        Orientation::UpNorth => (Facing::Up, Facing::North),
        Orientation::UpSouth => (Facing::Up, Facing::South),
        Orientation::UpWest => (Facing::Up, Facing::West),
        Orientation::WestUp => (Facing::West, Facing::Up),
        Orientation::EastUp => (Facing::East, Facing::Up),
        Orientation::NorthUp => (Facing::North, Facing::Up),
        Orientation::SouthUp => (Facing::South, Facing::Up),
    };
    let f = rotation.rotate_facing(mirror.mirror_facing(front));
    let t = rotation.rotate_facing(mirror.mirror_facing(top));
    match (f, t) {
        (Facing::Down, Facing::East) => Orientation::DownEast,
        (Facing::Down, Facing::North) => Orientation::DownNorth,
        (Facing::Down, Facing::South) => Orientation::DownSouth,
        (Facing::Down, Facing::West) => Orientation::DownWest,
        (Facing::Up, Facing::East) => Orientation::UpEast,
        (Facing::Up, Facing::North) => Orientation::UpNorth,
        (Facing::Up, Facing::South) => Orientation::UpSouth,
        (Facing::Up, Facing::West) => Orientation::UpWest,
        (Facing::West, Facing::Up) => Orientation::WestUp,
        (Facing::East, Facing::Up) => Orientation::EastUp,
        (Facing::North, Facing::Up) => Orientation::NorthUp,
        (Facing::South, Facing::Up) => Orientation::SouthUp,
        _ => orientation,
    }
}

const fn transform_cardinal_bool(
    dir: Facing,
    val: bool,
    rotation: Rotation,
    mirror: Mirror,
) -> Property {
    match rotation.rotate_facing(mirror.mirror_facing(dir)) {
        Facing::North => Property::North(val),
        Facing::South => Property::South(val),
        Facing::East => Property::East(val),
        _ => Property::West(val),
    }
}

fn transform_cardinal_wall(
    dir: Facing,
    index: u16,
    rotation: Rotation,
    mirror: Mirror,
) -> Property {
    match rotation.rotate_facing(mirror.mirror_facing(dir)) {
        Facing::North => Property::NorthWall(NorthWall::from_index(index)),
        Facing::South => Property::SouthWall(SouthWall::from_index(index)),
        Facing::East => Property::EastWall(EastWall::from_index(index)),
        _ => Property::WestWall(WestWall::from_index(index)),
    }
}

fn transform_cardinal_redstone(
    dir: Facing,
    index: u16,
    rotation: Rotation,
    mirror: Mirror,
) -> Property {
    match rotation.rotate_facing(mirror.mirror_facing(dir)) {
        Facing::North => Property::NorthRedstone(NorthRedstone::from_index(index)),
        Facing::South => Property::SouthRedstone(SouthRedstone::from_index(index)),
        Facing::East => Property::EastRedstone(EastRedstone::from_index(index)),
        _ => Property::WestRedstone(WestRedstone::from_index(index)),
    }
}

/// Transforms block state properties based on rotation and mirror, matching vanilla Minecraft.
///
/// Vanilla executes mirror first, then rotation.
#[must_use]
pub fn transform_block_properties(
    block_name: &str,
    properties: &[Property],
    rotation: Rotation,
    mirror: Mirror,
) -> Vec<Property> {
    if rotation == Rotation::None && mirror == Mirror::None {
        return properties.to_vec();
    }

    let is_stairs = block_name.ends_with("_stairs");
    let is_door = block_name.ends_with("_door") && !block_name.ends_with("trapdoor");
    let is_rail = block_name == "minecraft:rail"
        || block_name == "rail"
        || block_name == "minecraft:powered_rail"
        || block_name == "powered_rail"
        || block_name == "minecraft:detector_rail"
        || block_name == "detector_rail"
        || block_name == "minecraft:activator_rail"
        || block_name == "activator_rail";

    if is_stairs {
        let mut facing_opt = None;
        let mut shape_opt = None;
        for prop in properties {
            match *prop {
                Property::Facing(f) => facing_opt = Some(f),
                Property::StairsShape(s) => shape_opt = Some(s),
                _ => {}
            }
        }

        let (new_facing, new_shape) = if let Some(mut f) = facing_opt {
            let mut s = shape_opt.unwrap_or(StairsShape::Straight);

            // Step 1: StairBlock.mirror
            match mirror {
                Mirror::LeftRight => {
                    if matches!(f, Facing::North | Facing::South) {
                        f = match f {
                            Facing::North => Facing::South,
                            Facing::South => Facing::North,
                            _ => f,
                        };
                        s = match s {
                            StairsShape::Straight => StairsShape::Straight,
                            StairsShape::OuterLeft => StairsShape::OuterRight,
                            StairsShape::InnerRight => StairsShape::InnerLeft,
                            StairsShape::InnerLeft => StairsShape::InnerRight,
                            StairsShape::OuterRight => StairsShape::OuterLeft,
                        };
                    }
                }
                Mirror::FrontBack => {
                    if matches!(f, Facing::East | Facing::West) {
                        f = match f {
                            Facing::East => Facing::West,
                            Facing::West => Facing::East,
                            _ => f,
                        };
                        s = match s {
                            StairsShape::Straight => StairsShape::Straight,
                            StairsShape::OuterLeft => StairsShape::OuterRight,
                            StairsShape::InnerRight => StairsShape::InnerRight,
                            StairsShape::InnerLeft => StairsShape::InnerLeft,
                            StairsShape::OuterRight => StairsShape::OuterLeft,
                        };
                    }
                }
                Mirror::None => {}
            }

            // Step 2: StairBlock.rotate (only rotates facing)
            f = rotation.rotate_facing(f);

            (Some(f), Some(s))
        } else {
            (None, None)
        };

        return properties
            .iter()
            .map(|prop| match *prop {
                Property::Facing(_) if new_facing.is_some() => {
                    Property::Facing(new_facing.unwrap_or(Facing::North))
                }
                Property::StairsShape(_) if new_shape.is_some() => {
                    Property::StairsShape(new_shape.unwrap_or(StairsShape::Straight))
                }
                other => other,
            })
            .collect();
    }

    if is_door {
        let mut facing_opt = None;
        let mut hinge_opt = None;
        for prop in properties {
            match *prop {
                Property::Facing(f) => facing_opt = Some(f),
                Property::Hinge(h) => hinge_opt = Some(h),
                _ => {}
            }
        }

        let (new_facing, new_hinge) = if let Some(mut f) = facing_opt {
            let mut h = hinge_opt.unwrap_or(DoorHinge::Left);

            // Step 1: DoorBlock.mirror
            if mirror != Mirror::None {
                f = mirror.mirror_facing(f);
                h = match h {
                    DoorHinge::Left => DoorHinge::Right,
                    DoorHinge::Right => DoorHinge::Left,
                };
            }

            // Step 2: DoorBlock.rotate
            f = rotation.rotate_facing(f);

            (Some(f), Some(h))
        } else {
            (None, None)
        };

        return properties
            .iter()
            .map(|prop| match *prop {
                Property::Facing(_) if new_facing.is_some() => {
                    Property::Facing(new_facing.unwrap_or(Facing::North))
                }
                Property::Hinge(_) if new_hinge.is_some() => {
                    Property::Hinge(new_hinge.unwrap_or(DoorHinge::Left))
                }
                other => other,
            })
            .collect();
    }

    if is_rail {
        return properties
            .iter()
            .map(|prop| match *prop {
                Property::RailShape(s) => {
                    Property::RailShape(transform_rail_shape(s, rotation, mirror))
                }
                other => other,
            })
            .collect();
    }

    // General property transformation
    properties
        .iter()
        .map(|prop| match *prop {
            Property::North(v) => transform_cardinal_bool(Facing::North, v, rotation, mirror),
            Property::South(v) => transform_cardinal_bool(Facing::South, v, rotation, mirror),
            Property::East(v) => transform_cardinal_bool(Facing::East, v, rotation, mirror),
            Property::West(v) => transform_cardinal_bool(Facing::West, v, rotation, mirror),
            Property::NorthWall(v) => {
                transform_cardinal_wall(Facing::North, v.to_index(), rotation, mirror)
            }
            Property::SouthWall(v) => {
                transform_cardinal_wall(Facing::South, v.to_index(), rotation, mirror)
            }
            Property::EastWall(v) => {
                transform_cardinal_wall(Facing::East, v.to_index(), rotation, mirror)
            }
            Property::WestWall(v) => {
                transform_cardinal_wall(Facing::West, v.to_index(), rotation, mirror)
            }
            Property::NorthRedstone(v) => {
                transform_cardinal_redstone(Facing::North, v.to_index(), rotation, mirror)
            }
            Property::SouthRedstone(v) => {
                transform_cardinal_redstone(Facing::South, v.to_index(), rotation, mirror)
            }
            Property::EastRedstone(v) => {
                transform_cardinal_redstone(Facing::East, v.to_index(), rotation, mirror)
            }
            Property::WestRedstone(v) => {
                transform_cardinal_redstone(Facing::West, v.to_index(), rotation, mirror)
            }
            Property::Facing(f) => {
                Property::Facing(rotation.rotate_facing(mirror.mirror_facing(f)))
            }
            Property::Orientation(o) => {
                Property::Orientation(transform_orientation(o, rotation, mirror))
            }
            Property::Axis(a) => Property::Axis(rotation.rotate_axis(a)),
            Property::Rotation(r) => {
                let mirrored = mirror.mirror_block_rotation(i32::from(r));
                let rotated = rotation.rotate_block_rotation(mirrored);
                Property::Rotation(rotated as u8)
            }
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stairs_mirror_left_right() {
        // LeftRight inverts Z (North <-> South)
        let props = [
            Property::Facing(Facing::North),
            Property::StairsShape(StairsShape::InnerLeft),
        ];
        let transformed = transform_block_properties(
            "minecraft:oak_stairs",
            &props,
            Rotation::None,
            Mirror::LeftRight,
        );
        assert!(transformed.contains(&Property::Facing(Facing::South)));
        assert!(transformed.contains(&Property::StairsShape(StairsShape::InnerRight)));

        // LeftRight does not change East/West facing stairs shape or facing
        let props = [
            Property::Facing(Facing::East),
            Property::StairsShape(StairsShape::InnerLeft),
        ];
        let transformed = transform_block_properties(
            "minecraft:oak_stairs",
            &props,
            Rotation::None,
            Mirror::LeftRight,
        );
        assert!(transformed.contains(&Property::Facing(Facing::East)));
        assert!(transformed.contains(&Property::StairsShape(StairsShape::InnerLeft)));
    }

    #[test]
    fn stairs_mirror_front_back() {
        // FrontBack inverts X (East <-> West)
        let props = [
            Property::Facing(Facing::East),
            Property::StairsShape(StairsShape::OuterLeft),
        ];
        let transformed = transform_block_properties(
            "minecraft:oak_stairs",
            &props,
            Rotation::None,
            Mirror::FrontBack,
        );
        assert!(transformed.contains(&Property::Facing(Facing::West)));
        assert!(transformed.contains(&Property::StairsShape(StairsShape::OuterRight)));

        // FrontBack does not change North/South facing stairs
        let props = [
            Property::Facing(Facing::North),
            Property::StairsShape(StairsShape::OuterLeft),
        ];
        let transformed = transform_block_properties(
            "minecraft:oak_stairs",
            &props,
            Rotation::None,
            Mirror::FrontBack,
        );
        assert!(transformed.contains(&Property::Facing(Facing::North)));
        assert!(transformed.contains(&Property::StairsShape(StairsShape::OuterLeft)));
    }

    #[test]
    fn stairs_mirror_and_rotate() {
        // Mirror LeftRight then Rotate Clockwise90
        let props = [
            Property::Facing(Facing::North),
            Property::StairsShape(StairsShape::InnerLeft),
        ];
        let transformed = transform_block_properties(
            "minecraft:oak_stairs",
            &props,
            Rotation::Clockwise90,
            Mirror::LeftRight,
        );
        // north -> south (via LeftRight mirror) -> west (via Clockwise90 rotation)
        assert!(transformed.contains(&Property::Facing(Facing::West)));
        // inner_left -> inner_right (via LeftRight mirror, unchanged by rotation)
        assert!(transformed.contains(&Property::StairsShape(StairsShape::InnerRight)));
    }

    #[test]
    fn door_mirror_and_rotate() {
        let props = [
            Property::Facing(Facing::East),
            Property::Hinge(DoorHinge::Left),
        ];
        let transformed = transform_block_properties(
            "minecraft:oak_door",
            &props,
            Rotation::None,
            Mirror::FrontBack,
        );
        assert!(transformed.contains(&Property::Facing(Facing::West)));
        assert!(transformed.contains(&Property::Hinge(DoorHinge::Right)));
    }

    #[test]
    fn rail_mirror_and_rotate() {
        let props = [Property::RailShape(RailShape::NorthEast)];
        let transformed = transform_block_properties(
            "minecraft:rail",
            &props,
            Rotation::Clockwise90,
            Mirror::LeftRight,
        );
        assert!(transformed.contains(&Property::RailShape(RailShape::SouthWest)));
    }
}
