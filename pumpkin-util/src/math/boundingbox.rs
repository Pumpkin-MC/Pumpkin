use super::{position::BlockPos, vector3::Vector3};

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub min: Vector3<f64>,
    pub max: Vector3<f64>,
}

impl BoundingBox {
    pub fn new_default(size: &BoundingBoxSize) -> Self {
        Self::new_from_pos(Vector3::new(0.0, 0.0, 0.0), size)
    }

    pub fn new_from_pos(position: Vector3<f64>, size: &BoundingBoxSize) -> Self {
        let f = size.width / 2.0;
        Self {
            min: Vector3::new(position.x - f, position.y, position.z - f),
            max: Vector3::new(position.x + f, position.y + size.height, position.z + f),
        }
    }

    pub fn new(min: Vector3<f64>, max: Vector3<f64>) -> Self {
        Self { min, max }
    }

    pub fn from_block(position: &BlockPos) -> Self {
        let position = position.0;
        Self {
            min: Vector3::new(position.x as f64, position.y as f64, position.z as f64),
            max: Vector3::new(
                (position.x + 1) as f64,
                (position.y + 1) as f64,
                (position.z + 1) as f64,
            ),
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    pub fn intersects_block(&self, position: &BlockPos, bounding_box: &[f32]) -> bool {
        for i in 0..bounding_box.len() / 6 {
            let other = BoundingBox {
                min: Vector3::new(
                    position.0.x as f64 + bounding_box[i * 6] as f64,
                    position.0.y as f64 + bounding_box[i * 6 + 1] as f64,
                    position.0.z as f64 + bounding_box[i * 6 + 2] as f64,
                ),
                max: Vector3::new(
                    position.0.x as f64 + bounding_box[i * 6 + 3] as f64,
                    position.0.y as f64 + bounding_box[i * 6 + 4] as f64,
                    position.0.z as f64 + bounding_box[i * 6 + 5] as f64,
                ),
            };
            if self.intersects(&other) {
                return true;
            }
        }
        false
    }

    pub fn squared_magnitude(&self, pos: Vector3<f64>) -> f64 {
        let d = f64::max(f64::max(self.min.x - pos.x, pos.x - self.max.x), 0.0);
        let e = f64::max(f64::max(self.min.y - pos.y, pos.y - self.max.y), 0.0);
        let f = f64::max(f64::max(self.min.z - pos.z, pos.z - self.max.z), 0.0);
        super::squared_magnitude(d, e, f)
    }
}

#[derive(Clone, Copy)]
pub struct BoundingBoxSize {
    pub width: f64,
    pub height: f64,
}
