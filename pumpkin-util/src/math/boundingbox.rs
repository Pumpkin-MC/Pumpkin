use crate::math::{vector2::Vector2, vector3::Axis};

use super::{position::BlockPos, vector3::Vector3};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Vector3<f64>,
    pub max: Vector3<f64>,
}

#[derive(Clone, Copy, Debug)]

struct BoundingPlane {
    pub min: Vector2<f64>,
    pub max: Vector2<f64>,
}

impl BoundingPlane {
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    // Projecting a 3D box into 2D
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
    #[must_use]
    pub fn new_default(size: &EntityDimensions) -> Self {
        Self::new_from_pos(0., 0., 0., size)
    }

    #[must_use]
    pub fn new_from_pos(x: f64, y: f64, z: f64, size: &EntityDimensions) -> Self {
        let f = f64::from(size.width) / 2.;
        Self {
            min: Vector3::new(x - f, y, z - f),
            max: Vector3::new(x + f, y + f64::from(size.height), z + f),
        }
    }

    #[must_use]
    pub fn expand(&self, x: f64, y: f64, z: f64) -> Self {
        Self {
            min: Vector3::new(self.min.x - x, self.min.y - y, self.min.z - z),
            max: Vector3::new(self.max.x + x, self.max.y + y, self.max.z + z),
        }
    }

    #[must_use]
    pub fn expand_all(&self, value: f64) -> Self {
        self.expand(value, value, value)
    }

    #[must_use]
    pub fn contract_all(&self, value: f64) -> Self {
        self.expand_all(-value)
    }

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

    #[must_use]
    pub fn offset(&self, other: Self) -> Self {
        Self {
            min: self.min.add(&other.min),
            max: self.max.add(&other.max),
        }
    }

    #[must_use]
    pub const fn new(min: Vector3<f64>, max: Vector3<f64>) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn new_array(min: [f64; 3], max: [f64; 3]) -> Self {
        Self {
            min: Vector3::new(min[0], min[1], min[2]),
            max: Vector3::new(max[0], max[1], max[2]),
        }
    }

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

    #[must_use]
    pub const fn get_side(&self, max: bool) -> Vector3<f64> {
        if max { self.max } else { self.min }
    }

    #[must_use]
    pub fn calculate_collision_time(
        &self,
        other: &Self,
        movement: Vector3<f64>,
        axis: Axis,
        max_time: f64, // Start with 1.0
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

    #[must_use]
    pub fn get_average_side_length(&self) -> f64 {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        let depth = self.max.z - self.min.z;

        (width + height + depth) / 3.0
    }

    #[must_use]
    pub fn min_block_pos(&self) -> BlockPos {
        BlockPos::floored_v(self.min)
    }

    #[must_use]
    pub fn max_block_pos(&self) -> BlockPos {
        BlockPos::ceiled_v(self.max)
    }

    #[must_use]
    pub fn shift(&self, delta: Vector3<f64>) -> Self {
        Self {
            min: self.min + delta,

            max: self.max + delta,
        }
    }

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

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    #[must_use]
    pub fn squared_magnitude(&self, pos: Vector3<f64>) -> f64 {
        let d = f64::max(f64::max(self.min.x - pos.x, pos.x - self.max.x), 0.0);
        let e = f64::max(f64::max(self.min.y - pos.y, pos.y - self.max.y), 0.0);
        let f = f64::max(f64::max(self.min.z - pos.z, pos.z - self.max.z), 0.0);
        super::squared_magnitude(d, e, f)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntityDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

impl EntityDimensions {
    #[must_use]
    pub const fn new(width: f32, height: f32, eye_height: f32) -> Self {
        Self {
            width,
            height,
            eye_height,
        }
    }
}
