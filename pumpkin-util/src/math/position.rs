use super::{
    get_section_cord,
    vector3::{self, Vector3},
};
use std::fmt;
use std::hash::Hash;

use crate::math::vector2::Vector2;
use num_traits::Euclid;
use serde::{Deserialize, Serialize};

pub struct BlockPosIterator {
    start_x: i32,
    start_y: i32,
    start_z: i32,
    end_x: i32,
    end_y: i32,
    index: usize,
    count: usize,
}

impl BlockPosIterator {
    pub fn new(
        start_x: i32,
        start_y: i32,
        start_z: i32,
        end_x: i32,
        end_y: i32,
        end_z: i32,
    ) -> Self {
        let count_x = end_x - start_x + 1;
        let count_y = end_y - start_y + 1;
        let count_z = end_z - start_z + 1;
        let count = (count_x * count_y * count_z) as usize;
        BlockPosIterator {
            start_x,
            start_y,
            start_z,
            end_x,
            end_y,
            index: 0,
            count,
        }
    }
}

impl Iterator for BlockPosIterator {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let size_x = (self.end_x - self.start_x + 1) as usize;
        let size_y = (self.end_y - self.start_y + 1) as usize;

        let x_offset = self.index % size_x;
        let y_offset = (self.index / size_x) % size_y;
        let z_offset = (self.index / size_x) / size_y;

        let x = self.start_x + x_offset as i32;
        let y = self.start_y + y_offset as i32;
        let z = self.start_z + z_offset as i32;

        self.index += 1;
        Some(BlockPos::new(x, y, z))
    }
}

pub struct OutwardIterator {
    center_x: i32,
    center_y: i32,
    center_z: i32,
    range_x: i32,
    range_y: i32,
    range_z: i32,
    max_manhattan_distance: i32,

    pos: BlockPos,
    manhattan_distance: i32,
    limit_x: i32,
    limit_y: i32,
    dx: i32,
    dy: i32,
    swap_z: bool,
}

impl OutwardIterator {
    pub fn new(center: BlockPos, range_x: i32, range_y: i32, range_z: i32) -> Self {
        let max_manhattan_distance = range_x + range_y + range_z;
        OutwardIterator {
            center_x: center.0.x,
            center_y: center.0.y,
            center_z: center.0.z,
            range_x,
            range_y,
            range_z,
            max_manhattan_distance,
            pos: BlockPos::ZERO,
            manhattan_distance: 0,
            limit_x: 0,
            limit_y: 0,
            dx: 0,
            dy: 0,
            swap_z: false,
        }
    }
}

impl Iterator for OutwardIterator {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.swap_z {
            self.swap_z = false;
            self.pos.0.z = self.center_z - (self.pos.0.z - self.center_z);
            return Some(self.pos);
        }

        loop {
            if self.dy > self.limit_y {
                self.dx += 1;
                if self.dx > self.limit_x {
                    self.manhattan_distance += 1;
                    if self.manhattan_distance > self.max_manhattan_distance {
                        return None; // endOfData()
                    }
                    self.limit_x = self.range_x.min(self.manhattan_distance);
                    self.dx = -self.limit_x;
                }
                self.limit_y = self.range_y.min(self.manhattan_distance - self.dx.abs());
                self.dy = -self.limit_y;
            }

            let i2 = self.dx;
            let j2 = self.dy;
            let k2 = self.manhattan_distance - i2.abs() - j2.abs();

            if k2 <= self.range_z {
                self.swap_z = k2 != 0;
                self.pos =
                    BlockPos::new(self.center_x + i2, self.center_y + j2, self.center_z + k2);
                self.dy += 1;
                return Some(self.pos);
            }
            self.dy += 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// Aka Block Position
pub struct BlockPos(pub Vector3<i32>);

impl BlockPos {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(Vector3::new(x, y, z))
    }

    /// Iterates through all `BlockPos` within a cuboid region defined by two corner points,
    /// `start` and `end`. The iteration covers all blocks inclusively between the minimum
    /// and maximum coordinates of the two provided positions.
    ///
    /// This function ensures that the iteration always proceeds from the smallest to the
    /// largest coordinate in each dimension, regardless of whether `start` or `end` has a
    /// higher value for a given axis.
    ///
    /// # Arguments
    ///
    /// * `start` - One corner of the cuboid region.
    /// * `end` - The opposite corner of the cuboid region.
    ///
    /// # Returns
    ///
    /// A `BlockPosIterator` that yields each `BlockPos` within the defined cuboid.
    pub fn iterate(start: BlockPos, end: BlockPos) -> BlockPosIterator {
        BlockPosIterator::new(
            start.0.x.min(end.0.x),
            start.0.y.min(end.0.y),
            start.0.z.min(end.0.z),
            start.0.x.max(end.0.x),
            start.0.y.max(end.0.y),
            start.0.z.max(end.0.z),
        )
    }

    // Iterates through `BlockPos` objects outward from a specified `center` point.
    /// The iteration order is primarily based on the **Manhattan distance** from the center.
    ///
    /// For positions with the same Manhattan distance, the iteration prioritizes:
    /// 1.  **Y-offset**: From negative to positive.
    /// 2.  **X-offset**: From negative to positive (for the same Y-offset).
    /// 3.  **Z-offset**: Positive Z-offset before negative Z-offset (for the same X and Y offsets).
    ///
    /// # Arguments
    ///
    /// * `center` - The central `BlockPos` from which to start iterating.
    /// * `range_x` - The maximum absolute difference allowed in the X-coordinate from the center.
    /// * `range_y` - The maximum absolute difference allowed in the Y-coordinate from the center.
    /// * `range_z` - The maximum absolute difference allowed in the Z-coordinate from the center.
    ///
    /// # Returns
    ///
    /// An `OutwardIterator` that yields `BlockPos` instances in the described outward order.
    pub fn iterate_outwards(
        center: BlockPos,
        range_x: i32,
        range_y: i32,
        range_z: i32,
    ) -> OutwardIterator {
        OutwardIterator::new(center, range_x, range_y, range_z)
    }

    pub fn iterate_block_pos(
        start_x: i32,
        start_y: i32,
        start_z: i32,
        end_x: i32,
        end_y: i32,
        end_z: i32,
    ) -> BlockPosIterator {
        BlockPosIterator::new(start_x, start_y, start_z, end_x, end_y, end_z)
    }

    pub fn chunk_and_chunk_relative_position(&self) -> (Vector2<i32>, Vector3<i32>) {
        let (z_chunk, z_rem) = self.0.z.div_rem_euclid(&16);
        let (x_chunk, x_rem) = self.0.x.div_rem_euclid(&16);
        let chunk_coordinate = Vector2 {
            x: x_chunk,
            z: z_chunk,
        };

        // Since we divide by 16, remnant can never exceed u8
        let relative = Vector3 {
            x: x_rem,
            z: z_rem,

            y: self.0.y,
        };
        (chunk_coordinate, relative)
    }
    pub fn section_relative_position(&self) -> Vector3<i32> {
        let (_z_chunk, z_rem) = self.0.z.div_rem_euclid(&16);
        let (_x_chunk, x_rem) = self.0.x.div_rem_euclid(&16);
        let (_y_chunk, y_rem) = self.0.y.div_rem_euclid(&16);

        // Since we divide by 16 remnant can never exceed u8
        Vector3 {
            x: x_rem,
            z: z_rem,
            y: y_rem,
        }
    }
    pub fn from_i64(encoded_position: i64) -> Self {
        BlockPos(Vector3 {
            x: (encoded_position >> 38) as i32,
            y: (encoded_position << 52 >> 52) as i32,
            z: (encoded_position << 26 >> 38) as i32,
        })
    }

    pub fn floored(pos: Vector3<f64>) -> Self {
        Self(Vector3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        ))
    }

    pub fn ceiled(pos: Vector3<f64>) -> Self {
        Self(Vector3::new(
            pos.x.ceil() as i32,
            pos.y.ceil() as i32,
            pos.z.ceil() as i32,
        ))
    }

    pub fn to_f64(&self) -> Vector3<f64> {
        Vector3::new(
            self.0.x as f64 + 0.5,
            self.0.y as f64,
            self.0.z as f64 + 0.5,
        )
    }

    pub fn to_centered_f64(&self) -> Vector3<f64> {
        Vector3::new(
            self.0.x as f64 + 0.5,
            self.0.y as f64 + 0.5,
            self.0.z as f64 + 0.5,
        )
    }

    pub fn offset(&self, offset: Vector3<i32>) -> Self {
        BlockPos(self.0 + offset)
    }

    pub fn add(&self, x: i32, y: i32, z: i32) -> Self {
        BlockPos::new(self.0.x + x, self.0.y + y, self.0.z + z)
    }

    pub fn offset_dir(&self, offset: Vector3<i32>, direction: i32) -> Self {
        BlockPos(Vector3::new(
            self.0.x + offset.x * direction,
            self.0.y + offset.y * direction,
            self.0.z + offset.z * direction,
        ))
    }

    pub fn up(&self) -> Self {
        self.offset(Vector3::new(0, 1, 0))
    }

    pub fn up_height(&self, height: i32) -> Self {
        self.offset(Vector3::new(0, height, 0))
    }

    pub fn down(&self) -> Self {
        self.offset(Vector3::new(0, -1, 0))
    }

    pub fn down_height(&self, height: i32) -> Self {
        self.offset(Vector3::new(0, -height, 0))
    }

    pub fn manhattan_distance(&self, other: Self) -> i32 {
        let x = (other.0.x - self.0.x).abs();
        let y = (other.0.y - self.0.y).abs();
        let z = (other.0.z - self.0.z).abs();
        x + y + z
    }
}

impl Serialize for BlockPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let long = ((self.0.x as i64 & 0x3FFFFFF) << 38)
            | ((self.0.z as i64 & 0x3FFFFFF) << 12)
            | (self.0.y as i64 & 0xFFF);
        serializer.serialize_i64(long)
    }
}

impl<'de> Deserialize<'de> for BlockPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = BlockPos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("An i64 int")
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BlockPos(Vector3 {
                    x: (v >> 38) as i32,
                    y: (v << 52 >> 52) as i32,
                    z: (v << 26 >> 38) as i32,
                }))
            }
        }
        deserializer.deserialize_i64(Visitor)
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.0.x, self.0.y, self.0.z)
    }
}

#[must_use]
pub const fn chunk_section_from_pos(block_pos: &BlockPos) -> Vector3<i32> {
    let block_pos = block_pos.0;
    Vector3::new(
        get_section_cord(block_pos.x),
        get_section_cord(block_pos.y),
        get_section_cord(block_pos.z),
    )
}

pub const fn get_local_cord(cord: i32) -> i32 {
    cord & 15
}

#[must_use]
pub fn pack_local_chunk_section(block_pos: &BlockPos) -> i16 {
    let x = get_local_cord(block_pos.0.x);
    let z = get_local_cord(block_pos.0.z);
    let y = get_local_cord(block_pos.0.y);
    vector3::packed_local(&Vector3::new(x, y, z))
}
