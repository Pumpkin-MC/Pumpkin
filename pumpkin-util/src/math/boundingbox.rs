use crate::math::{vector2::Vector2, vector3::Axis};

use super::{position::BlockPos, vector3::Vector3};

/// Represents an axis-aligned bounding box in 3D space.
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    /// The minimum corner of the box.
    pub min: Vector3<f64>,
    /// The maximum corner of the box.
    pub max: Vector3<f64>,
}

/// Represents a 2D bounding plane used for collision checks.
#[derive(Clone, Copy, Debug)]
struct BoundingPlane {
    /// The minimum corner of the plane.
    pub min: Vector2<f64>,
    /// The maximum corner of the plane.
    pub max: Vector2<f64>,
}

impl BoundingPlane {
    /// Checks whether this plane intersects another plane.
    ///
    /// # Arguments
    /// * `other` – The other bounding plane to check against.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Projects a 3D bounding box onto a 2D plane by excluding one axis.
    ///
    /// # Arguments
    /// * `bounding_box` – The 3D bounding box to project.
    /// * `excluded` – The axis to exclude from the projection.
    pub const fn from_box(bounding_box: &BoundingBox, excluded: Axis) -> Self {
        let [axis1, axis2] = Axis::excluding(excluded);

        Self {
            min: Vector2::new(
                bounding_box.get_side(false).get_axis(axis1),
                bounding_box.get_side(false).get_axis(axis2),
            ),

            max: Vector2::new(
                bounding_box.get_side(true).get_axis(axis1),
                bounding_box.get_side(true).get_axis(axis2),
            ),
        }
    }
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

    /// Calculates the collision time with another bounding box along a movement vector.
    ///
    /// # Arguments
    /// * `other` – The bounding box to test collision against.
    /// * `movement` – Movement vector of this box.
    /// * `axis` – Axis along which to calculate collision.
    /// * `max_time` – Maximum allowed collision time.
    ///
    /// # Returns
    /// Some(f64) if a collision occurs within `max_time`, None otherwise.
    #[must_use]
    pub fn calculate_collision_time(
        &self,
        other: &Self,
        movement: Vector3<f64>,
        axis: Axis,
        max_time: f64, // NOTE: Start with 1.0
    ) -> Option<f64> {
        let movement_on_axis = movement.get_axis(axis);

        if movement_on_axis == 0.0 {
            return None;
        }

        let move_positive = movement_on_axis.is_sign_positive();
        let self_plane_const = self.get_side(move_positive).get_axis(axis);
        let other_plane_const = other.get_side(!move_positive).get_axis(axis);
        let collision_time = (other_plane_const - self_plane_const) / movement_on_axis;

        if collision_time < 0.0 || collision_time >= max_time {
            return None;
        }

        let self_moved = self.shift(movement * collision_time);
        let self_plane_moved = BoundingPlane::from_box(&self_moved, axis);
        let other_plane = BoundingPlane::from_box(other, axis);

        if !self_plane_moved.intersects(&other_plane) {
            return None;
        }

        Some(collision_time)
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
    pub fn min_block_pos(&self) -> BlockPos {
        BlockPos::floored_v(self.min)
    }

    /// Returns the maximum block position covered by this bounding box.
    #[must_use]
    pub fn max_block_pos(&self) -> BlockPos {
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
    pub fn stretch(&self, other: Vector3<f64>) -> Self {
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
