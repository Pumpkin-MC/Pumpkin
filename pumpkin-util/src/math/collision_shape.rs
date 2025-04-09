use super::{boundingbox::BoundingBox, vector3::Vector3};

#[derive(Clone, Copy, Debug)]
pub struct CollisionShape {
    pub min: Vector3<f64>,
    pub max: Vector3<f64>,
}

impl CollisionShape {
    pub fn is_empty() -> bool {
        unimplemented!()
    }

    pub fn intersects(&self, bounding_box: &BoundingBox) -> bool {
        let min = self.min;
        let max = self.max;
        let box_min = bounding_box.min;
        let box_max = bounding_box.max;

        if min.x > box_max.x || max.x < box_min.x {
            return false;
        }
        if min.y > box_max.y || max.y < box_min.y {
            return false;
        }
        if min.z > box_max.z || max.z < box_min.z {
            return false;
        }
        true
    }
}
