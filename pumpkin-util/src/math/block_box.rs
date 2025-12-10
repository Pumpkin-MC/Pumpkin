use crate::math::{position::BlockPos, vector3::Vector3};

#[derive(Clone, Copy, Debug)]
pub struct BlockBox {
    pub min: Vector3<i32>,
    pub max: Vector3<i32>,
}

impl BlockBox {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        BlockBox {
            min: Vector3 {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            max: Vector3 {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        }
    }

    pub fn from_pos(pos: BlockPos) -> Self {
        Self {
            min: pos.0,
            max: pos.0,
        }
    }
}
