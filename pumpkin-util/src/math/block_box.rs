use crate::{
    BlockDirection,
    math::{
        position::BlockPos,
        vector3::{Axis, Vector3},
    },
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

    #[expect(clippy::too_many_arguments)]
    pub fn rotated(
        x: i32,
        y: i32,
        z: i32,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        size_x: i32,
        size_y: i32,
        size_z: i32,
        facing: &BlockDirection,
    ) -> Self {
        match facing {
            BlockDirection::North => BlockBox::new(
                x + offset_x,
                y + offset_y,
                z - size_z + 1 + offset_z,
                x + size_x - 1 + offset_x,
                y + size_y - 1 + offset_y,
                z + offset_z,
            ),
            BlockDirection::West => BlockBox::new(
                x - size_z + 1 + offset_z,
                y + offset_y,
                z + offset_x,
                x + offset_z,
                y + size_y - 1 + offset_y,
                z + size_x - 1 + offset_x,
            ),
            BlockDirection::East => BlockBox::new(
                x + offset_z,
                y + offset_y,
                z + offset_x,
                x + size_z - 1 + offset_z,
                y + size_y - 1 + offset_y,
                z + size_x - 1 + offset_x,
            ),
            // Default / South
            _ => BlockBox::new(
                x + offset_x,
                y + offset_y,
                z + offset_z,
                x + size_x - 1 + offset_x,
                y + size_y - 1 + offset_y,
                z + size_z - 1 + offset_z,
            ),
        }
    }

    pub fn from_pos(pos: BlockPos) -> Self {
        Self {
            min: pos.0,
            max: pos.0,
        }
    }

    pub fn move_pos(&mut self, dx: i32, dy: i32, dz: i32) {
        self.min.x += dx;
        self.min.y += dy;
        self.min.z += dz;
        self.max.x += dx;
        self.max.y += dy;
        self.max.z += dz;
    }

    pub fn contains_pos(&self, pos: &Vector3<i32>) -> bool {
        self.contains(pos.x, pos.y, pos.z)
    }

    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min.x
            && x <= self.max.x
            && y >= self.min.y
            && y <= self.max.y
            && z >= self.min.z
            && z <= self.max.z
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    pub fn intersects_xz(&self, other: &Self) -> bool {
        self.max.x >= other.min.x
            && self.min.x <= other.max.x
            && self.max.z >= other.min.z
            && self.min.z <= other.max.z
    }

    pub fn intersects_raw_xz(&self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> bool {
        self.max.x >= min_x && self.min.x <= max_x && self.max.z >= min_z && self.min.z <= max_z
    }

    pub fn get_block_count_y(&self) -> i32 {
        self.max.y - self.min.y + 1
    }

    pub fn encompass(&mut self, other: &BlockBox) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
    }

    /// Static helper to find the box covering a collection of boxes
    pub fn encompass_all<I>(boxes: I) -> Option<BlockBox>
    where
        I: IntoIterator<Item = BlockBox>,
    {
        let mut iter = boxes.into_iter();
        let mut result = iter.next()?; // Return None if empty

        for b in iter {
            result.encompass(&b);
        }
        Some(result)
    }
}
