use crate::{
    BlockDirection,
    math::{
        position::BlockPos,
        vector3::{Axis, Vector3},
    },
};

/// Represents an axis-aligned 3D block bounding box in integer coordinates.
#[derive(Clone, Copy, Debug)]
pub struct BlockBox {
    /// The minimum corner of the box (inclusive).
    pub min: Vector3<i32>,
    /// The maximum corner of the box (inclusive).
    pub max: Vector3<i32>,
}

impl BlockBox {
    /// Creates a new box from min/max coordinates.
    ///
    /// # Arguments
    /// * `min_x` – Minimum X coordinate (inclusive).
    /// * `min_y` – Minimum Y coordinate (inclusive).
    /// * `min_z` – Minimum Z coordinate (inclusive).
    /// * `max_x` – Maximum X coordinate (inclusive).
    /// * `max_y` – Maximum Y coordinate (inclusive).
    /// * `max_z` – Maximum Z coordinate (inclusive).
    #[must_use]
    pub const fn new(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Self {
        Self {
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

    /// Creates a box along an axis with specified dimensions.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` – Starting coordinates.
    /// * `axis` – Axis along which the box extends.
    /// * `width`, `height`, `depth` – Dimensions of the box.
    #[must_use]
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

    /// Creates a rotated box from offsets and size.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` – Base coordinates.
    /// * `offset_x`, `offset_y`, `offset_z` – Offsets from base.
    /// * `size_x`, `size_y`, `size_z` – Dimensions.
    /// * `facing` – Facing direction.
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn rotated(
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
            BlockDirection::North => Self::new(
                x + offset_x,
                y + offset_y,
                z - size_z + 1 + offset_z,
                x + size_x - 1 + offset_x,
                y + size_y - 1 + offset_y,
                z + offset_z,
            ),
            BlockDirection::West => Self::new(
                x - size_z + 1 + offset_z,
                y + offset_y,
                z + offset_x,
                x + offset_z,
                y + size_y - 1 + offset_y,
                z + size_x - 1 + offset_x,
            ),
            BlockDirection::East => Self::new(
                x + offset_z,
                y + offset_y,
                z + offset_x,
                x + size_z - 1 + offset_z,
                y + size_y - 1 + offset_y,
                z + size_x - 1 + offset_x,
            ),
            // Default / South
            _ => Self::new(
                x + offset_x,
                y + offset_y,
                z + offset_z,
                x + size_x - 1 + offset_x,
                y + size_y - 1 + offset_y,
                z + size_z - 1 + offset_z,
            ),
        }
    }

    /// Creates a box covering a single block position.
    ///
    /// # Arguments
    /// * `pos` – Block position.
    #[must_use]
    pub const fn from_pos(pos: BlockPos) -> Self {
        Self {
            min: pos.0,
            max: pos.0,
        }
    }

    /// Expands the box in all directions.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` – Amount to expand on each axis.
    #[must_use]
    pub const fn expand(&self, x: i32, y: i32, z: i32) -> Self {
        Self {
            min: Vector3::new(self.min.x - x, self.min.y - y, self.min.z - z),
            max: Vector3::new(self.max.x + x, self.max.y + y, self.max.z + z),
        }
    }

    /// Moves the box by the given offsets.
    ///
    /// # Arguments
    /// * `dx`, `dy`, `dz` – Offset amounts.
    pub const fn move_pos(&mut self, dx: i32, dy: i32, dz: i32) {
        self.min.x += dx;
        self.min.y += dy;
        self.min.z += dz;
        self.max.x += dx;
        self.max.y += dy;
        self.max.z += dz;
    }

    /// Returns `true` if the box contains the given block position.
    ///
    /// # Arguments
    /// * `pos` – Block coordinates.
    #[must_use]
    pub const fn contains_pos(&self, pos: &Vector3<i32>) -> bool {
        self.contains(pos.x, pos.y, pos.z)
    }

    /// Returns `true` if the box contains the given coordinates.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` – Coordinates to test.
    #[must_use]
    pub const fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min.x
            && x <= self.max.x
            && y >= self.min.y
            && y <= self.max.y
            && z >= self.min.z
            && z <= self.max.z
    }

    /// Returns `true` if this box intersects another box in 3D.
    ///
    /// # Arguments
    /// * `other` – Other box to test.
    #[must_use]
    pub const fn intersects(&self, other: &Self) -> bool {
        self.max.x >= other.min.x
            && self.min.x <= other.max.x
            && self.max.z >= other.min.z
            && self.min.z <= other.max.z
            && self.max.y >= other.min.y
            && self.min.y <= other.max.y
    }

    /// Returns `true` if this box intersects another box in the XZ plane.
    ///
    /// # Arguments
    /// * `other` – Other box to test.
    #[must_use]
    pub const fn intersects_xz(&self, other: &Self) -> bool {
        self.max.x >= other.min.x
            && self.min.x <= other.max.x
            && self.max.z >= other.min.z
            && self.min.z <= other.max.z
    }

    /// Returns `true` if this box intersects given raw XZ coordinates.
    ///
    /// # Arguments
    /// * `min_x`, `min_z`, `max_x`, `max_z` – Raw XZ bounds.
    #[must_use]
    pub const fn intersects_raw_xz(&self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> bool {
        self.max.x >= min_x && self.min.x <= max_x && self.max.z >= min_z && self.min.z <= max_z
    }

    /// Returns the number of blocks along the Y axis.
    #[must_use]
    pub const fn get_block_count_y(&self) -> i32 {
        self.max.y - self.min.y + 1
    }

    /// Expands this box to encompass another box.
    ///
    /// # Arguments
    /// * `other` – Box to encompass.
    pub fn encompass(&mut self, other: &Self) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
    }

    /// Returns a box covering all boxes in an iterator.
    ///
    /// # Arguments
    /// * `boxes` – Iterator of boxes.
    /// # Returns
    /// * `Some(Box)` covering all boxes, or `None` if empty.
    pub fn encompass_all<I>(boxes: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let mut iter = boxes.into_iter();
        let mut result = iter.next()?; // NOTE: Return None if empty

        for b in iter {
            result.encompass(&b);
        }
        Some(result)
    }
}
