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

    pub fn at_pos(self, pos: BlockPos) -> BoundingBox {
        self.to_box().shift(pos.0.to_f64())
    }

    pub fn new(min: Vector3<f64>, max: Vector3<f64>) -> Self {
        Self { min, max }
    }

    fn to_box(self) -> BoundingBox {
        BoundingBox {
            min: self.min,
            max: self.max,
        }
    }
}
