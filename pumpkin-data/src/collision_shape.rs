use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

#[derive(Clone, Copy, Debug)]
pub struct CollisionShape {
    pub min: Vector3<f64>,
    pub max: Vector3<f64>,
}

impl CollisionShape {
    pub fn is_empty() -> bool {
        unimplemented!()
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    pub fn at_pos(&self, pos: BlockPos) -> Self {
        let vec3 = Vector3 {
            x: pos.0.x as f64,
            y: pos.0.y as f64,
            z: pos.0.z as f64,
        };
        Self {
            min: self.min + vec3,
            max: self.max + vec3,
        }
    }
    
    pub fn to_box(&self) -> BoundingBox {
        BoundingBox {
            min: self.min,
            max: self.max,
        }
    }
}
