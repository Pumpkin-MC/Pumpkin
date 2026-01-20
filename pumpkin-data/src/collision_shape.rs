use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

#[derive(Clone, Copy, Debug)]
pub struct CollisionShape {
    pub min: Vector3<f64>,
    pub max: Vector3<f64>,
}

impl CollisionShape {
    #[must_use] 
    pub const fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox {
            min: self.min,
            max: self.max,
        }
    }

    #[must_use] 
    pub const fn new(min: Vector3<f64>, max: Vector3<f64>) -> Self {
        Self { min, max }
    }

    #[must_use] 
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
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
}
