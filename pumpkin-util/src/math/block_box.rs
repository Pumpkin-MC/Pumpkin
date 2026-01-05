use crate::math::{
    position::BlockPos,
    vector3::{Axis, Vector3},
};

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
    pub fn create_box(
        x: i32,
        y: i32,
        z: i32,
        axis: Axis,
        width: i32,
        height: i32,
        depth: i32,
    ) -> Self {
        if axis == Axis::Z {
            Self::new(x, y, z, x + width - 1, y + height - 1, z + depth - 1)
        } else {
            Self::new(x, y, z, x + depth - 1, y + height - 1, z + width - 1)
        }
    }

    pub fn from_pos(pos: BlockPos) -> Self {
        Self {
            min: pos.0,
            max: pos.0,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}
