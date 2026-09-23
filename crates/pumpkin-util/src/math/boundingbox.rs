use crate::math::vector3::Axis;

use super::{position::BlockPos, vector3::Vector3};

/// Represents an axis-aligned bounding box in 3D space.
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    /// The minimum corner of the box.
    pub min: Vector3<f64>,
    /// The maximum corner of the box.
    pub max: Vector3<f64>,
}

impl BoundingBox {
    /// Creates a default bounding box at the origin using entity dimensions.
    ///
    /// # Arguments
    /// * `size` – Dimensions of the entity.
    #[must_use]
    pub fn new_default(size: &EntityDimensions) -> Self {
        Self::new_from_pos(0., 0., 0., size)
    }

    /// Creates a bounding box from a position and entity dimension.
    ///
    /// # Arguments
    /// * `x` – X coordinate of the position.
    /// * `y` – Y coordinate of the position.
    /// * `z` – Z coordinate of the position.
    /// * `size` – Dimensions of the entity.
    #[must_use]
    pub fn new_from_pos(x: f64, y: f64, z: f64, size: &EntityDimensions) -> Self {
        let f = f64::from(size.width) / 2.;
        Self {
            min: Vector3::new(x - f, y, z - f),
            max: Vector3::new(x + f, y + f64::from(size.height), z + f),
        }
    }

    /// Expands this box by given amounts along each axis.
    ///
    /// # Arguments
    /// * `x` – Amount to expand along the X axis.
    /// * `y` – Amount to expand along the Y axis.
    /// * `z` – Amount to expand along the Z axis.
    #[must_use]
    pub fn expand(&self, x: f64, y: f64, z: f64) -> Self {
        Self {
            min: Vector3::new(self.min.x - x, self.min.y - y, self.min.z - z),
            max: Vector3::new(self.max.x + x, self.max.y + y, self.max.z + z),
        }
    }

    /// Expands this bounding box towards a specific direction.
    ///
    /// If a provided value is negative, it extends the minimum boundary along that axis.
    /// If a provided value is positive, it extends the maximum boundary along that axis.
    ///
    /// # Arguments
    /// * `x` – Amount to expand towards on the X axis.
    /// * `y` – Amount to expand towards on the Y axis.
    /// * `z` – Amount to expand towards on the Z axis.
    #[must_use]
    pub fn expand_towards(&self, x: f64, y: f64, z: f64) -> Self {
        let mut min_x = self.min.x;
        let mut min_y = self.min.y;
        let mut min_z = self.min.z;

        let mut max_x = self.max.x;
        let mut max_y = self.max.y;
        let mut max_z = self.max.z;

        if x < 0.0 {
            min_x += x;
        } else if x > 0.0 {
            max_x += x;
        }

        if y < 0.0 {
            min_y += y;
        } else if y > 0.0 {
            max_y += y;
        }

        if z < 0.0 {
            min_z += z;
        } else if z > 0.0 {
            max_z += z;
        }

        Self {
            min: Vector3::new(min_x, min_y, min_z),
            max: Vector3::new(max_x, max_y, max_z),
        }
    }

    /// Expands this box uniformly along all axes.
    ///
    /// # Arguments
    /// * `value` – Amount to expand along all axes.
    #[must_use]
    pub fn expand_all(&self, value: f64) -> Self {
        self.expand(value, value, value)
    }

    /// Contracts this box uniformly along all axes.
    ///
    /// # Arguments
    /// * `value` – Amount to contract along all axes.
    #[must_use]
    pub fn contract_all(&self, value: f64) -> Self {
        self.expand_all(-value)
    }

    /// Returns a new bounding box shifted to a specific block position.
    ///
    /// # Arguments
    /// * `pos` – Block position to move the box to.
    #[must_use]
    pub fn at_pos(&self, pos: BlockPos) -> Self {
        let vec3 = Vector3 {
            x: f64::from(pos.0.x),
            y: f64::from(pos.0.y),
            z: f64::from(pos.0.z),
        };
        Self {
            min: self.min + vec3,
            max: self.max + vec3,
        }
    }

    /// Returns a new bounding box offset by another bounding box.
    ///
    /// # Arguments
    /// * `other` – The bounding box to add as an offset.
    #[must_use]
    pub fn offset(&self, other: Self) -> Self {
        Self {
            min: self.min.add(&other.min),
            max: self.max.add(&other.max),
        }
    }

    /// Creates a bounding box from explicit min and max coordinates.
    ///
    /// # Arguments
    /// * `min` – Minimum corner of the box.
    /// * `max` – Maximum corner of the box.
    #[must_use]
    pub const fn new(min: Vector3<f64>, max: Vector3<f64>) -> Self {
        Self { min, max }
    }

    /// Creates a bounding box from arrays of min and max coordinates.
    ///
    /// # Arguments
    /// * `min` – Minimum corner as an array [x, y, z].
    /// * `max` – Maximum corner as an array [x, y, z].
    #[must_use]
    pub const fn new_array(min: [f64; 3], max: [f64; 3]) -> Self {
        Self {
            min: Vector3::new(min[0], min[1], min[2]),
            max: Vector3::new(max[0], max[1], max[2]),
        }
    }

    /// Returns a bounding box representing a full block from (0,0,0) to (1,1,1).
    #[must_use]
    pub const fn full_block() -> Self {
        Self {
            min: Vector3::new(0f64, 0f64, 0f64),
            max: Vector3::new(1f64, 1f64, 1f64),
        }
    }

    /// Creates a bounding box from a block position covering a full block.
    ///
    /// # Arguments
    /// * `position` – Block position to base the bounding box on.
    #[must_use]
    pub fn from_block(position: &BlockPos) -> Self {
        let position = position.0;
        Self {
            min: Vector3::new(
                f64::from(position.x),
                f64::from(position.y),
                f64::from(position.z),
            ),
            max: Vector3::new(
                f64::from(position.x) + 1.0,
                f64::from(position.y) + 1.0,
                f64::from(position.z) + 1.0,
            ),
        }
    }

    /// Returns the min or max side of the bounding box.
    ///
    /// # Arguments
    /// * `max` – Whether to return the max side (true) or min side (false).
    #[must_use]
    pub const fn get_side(&self, max: bool) -> Vector3<f64> {
        if max { self.max } else { self.min }
    }

    /// Vanilla `Shapes.collide`: how far this box can move along `axis`, up to `distance`, before
    /// it touches one of `boxes`. A box overlapped by less than `1e-7` still blocks.
    ///
    /// # Returns
    /// The allowed distance and the index of the box that limited it, if any.
    #[must_use]
    pub fn collide_along(
        &self,
        axis: Axis,
        boxes: &[Self],
        mut distance: f64,
    ) -> (f64, Option<usize>) {
        const EPSILON: f64 = 1.0e-7;
        let [a, b] = Axis::excluding(axis);
        let mut blocker = None;

        for (i, other) in boxes.iter().enumerate() {
            if distance.abs() < EPSILON {
                return (0.0, blocker);
            }
            // Only boxes overlapping on the two other axes can block.
            if other.min.get_axis(a) >= self.max.get_axis(a) - EPSILON
                || other.max.get_axis(a) <= self.min.get_axis(a) + EPSILON
                || other.min.get_axis(b) >= self.max.get_axis(b) - EPSILON
                || other.max.get_axis(b) <= self.min.get_axis(b) + EPSILON
            {
                continue;
            }

            if distance > 0.0 {
                let gap = other.min.get_axis(axis) - self.max.get_axis(axis);
                if gap >= -EPSILON && gap < distance {
                    distance = gap;
                    blocker = Some(i);
                }
            } else {
                let gap = other.max.get_axis(axis) - self.min.get_axis(axis);
                if gap <= EPSILON && gap > distance {
                    distance = gap;
                    blocker = Some(i);
                }
            }
        }

        (distance, blocker)
    }

    /// Returns the average side length of the bounding box.
    #[must_use]
    pub fn get_average_side_length(&self) -> f64 {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        let depth = self.max.z - self.min.z;

        (width + height + depth) / 3.0
    }

    /// Returns the minimum block position covered by this bounding box.
    #[must_use]
    pub const fn min_block_pos(&self) -> BlockPos {
        BlockPos::floored_v(self.min)
    }

    /// Returns the maximum block position covered by this bounding box.
    #[must_use]
    pub const fn max_block_pos(&self) -> BlockPos {
        // Use a tiny epsilon and floor the max coordinates so that a box whose
        // max is exactly on a block boundary does not include the adjacent
        // block. This mirrors vanilla behaviour where max block is inclusive
        // only when the entity actually overlaps that block.
        let eps = 1e-9f64;
        BlockPos::floored_v(Vector3::new(
            self.max.x - eps,
            self.max.y - eps,
            self.max.z - eps,
        ))
    }

    /// Returns a new bounding box shifted by a delta vector.
    ///
    /// # Arguments
    /// * `delta` – Vector to shift the bounding box by.
    #[must_use]
    pub fn shift(&self, delta: Vector3<f64>) -> Self {
        Self {
            min: self.min + delta,
            max: self.max + delta,
        }
    }

    /// Stretches this bounding box along each axis by a given vector.
    ///
    /// # Arguments
    /// * `other` – Vector specifying how much to stretch along each axis.
    #[must_use]
    pub const fn stretch(&self, other: Vector3<f64>) -> Self {
        let mut new = *self;

        if other.x < 0.0 {
            new.min.x += other.x;
        } else if other.x > 0.0 {
            new.max.x += other.x;
        }

        if other.y < 0.0 {
            new.min.y += other.y;
        } else if other.y > 0.0 {
            new.max.y += other.y;
        }

        if other.z < 0.0 {
            new.min.z += other.z;
        } else if other.z > 0.0 {
            new.max.z += other.z;
        }

        new
    }

    /// Creates a bounding box from a block position with zero volume.
    ///
    /// # Arguments
    /// * `position` – Block position to base the bounding box on.
    #[must_use]
    pub fn from_block_raw(position: &BlockPos) -> Self {
        let position = position.0;
        Self {
            min: Vector3::new(
                f64::from(position.x),
                f64::from(position.y),
                f64::from(position.z),
            ),
            max: Vector3::new(
                f64::from(position.x),
                f64::from(position.y),
                f64::from(position.z),
            ),
        }
    }

    /// Checks if this bounding box intersects another bounding box.
    ///
    /// # Arguments
    /// * `other` – The other bounding box to check against.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    /// Computes the squared magnitude from a point to the nearest point on this bounding box.
    ///
    /// # Arguments
    /// * `pos` – The point to measure from.
    #[must_use]
    pub fn squared_magnitude(&self, pos: Vector3<f64>) -> f64 {
        let d = f64::max(f64::max(self.min.x - pos.x, pos.x - self.max.x), 0.0);
        let e = f64::max(f64::max(self.min.y - pos.y, pos.y - self.max.y), 0.0);
        let f = f64::max(f64::max(self.min.z - pos.z, pos.z - self.max.z), 0.0);

        super::squared_magnitude(d, e, f)
    }
}

/// Represents the dimensions of an entity.
#[derive(Clone, Copy, Debug)]
pub struct EntityDimensions {
    /// Width of the entity.
    pub width: f32,
    /// Height of the entity.
    pub height: f32,
    /// Eye height relative to the bottom of the entity.
    pub eye_height: f32,
}

impl EntityDimensions {
    /// Creates a new entity dimensions object.
    ///
    /// # Arguments
    /// * `width` – Width of the entity.
    /// * `height` – Height of the entity.
    /// * `eye_height` – Eye height of the entity.
    #[must_use]
    pub const fn new(width: f32, height: f32, eye_height: f32) -> Self {
        Self {
            width,
            height,
            eye_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundingBox;
    use crate::math::vector3::{Axis, Vector3};

    fn bb(min: (f64, f64, f64), max: (f64, f64, f64)) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(min.0, min.1, min.2),
            Vector3::new(max.0, max.1, max.2),
        )
    }

    #[test]
    fn collide_along_stops_at_floor_and_wall() {
        let floor = bb((0.0, 63.0, 0.0), (1.0, 64.0, 1.0));
        let item = bb((0.375, 64.2, 0.375), (0.625, 64.45, 0.625));
        let (dy, blocker) = item.collide_along(Axis::Y, &[floor], -0.5);
        assert!((dy + 0.2).abs() < 1e-9);
        assert_eq!(blocker, Some(0));

        let wall = bb((1.0, 64.0, 0.0), (2.0, 65.0, 1.0));
        let resting = bb((0.7, 64.0, 0.4), (0.95, 64.25, 0.65));
        let (dx, _) = resting.collide_along(Axis::X, &[wall], 0.3);
        assert!((dx - 0.05).abs() < 1e-9);
        // Only boxes overlapping on the other axes block.
        let (dz, blocker) = resting.collide_along(Axis::Z, &[wall], 0.3);
        assert_eq!((dz, blocker), (0.3, None));
    }

    #[test]
    fn collide_along_blocks_within_epsilon_overlap() {
        let floor = bb((0.0, 63.0, 0.0), (1.0, 64.0, 1.0));
        // Rounding left the item 5e-8 inside the floor: vanilla still treats it as resting.
        let sunk = bb((0.375, 64.0 - 5e-8, 0.375), (0.625, 64.25, 0.625));
        let (dy, _) = sunk.collide_along(Axis::Y, &[floor], -0.04);
        assert!(dy.abs() < 1e-7);
        // Deeper overlap is ignored, like vanilla.
        let inside = bb((0.375, 63.9, 0.375), (0.625, 64.15, 0.625));
        assert_eq!(
            inside.collide_along(Axis::Y, &[floor], -0.04),
            (-0.04, None)
        );
    }
}
