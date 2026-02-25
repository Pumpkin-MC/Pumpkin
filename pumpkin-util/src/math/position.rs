use super::{
    get_section_cord,
    vector3::{self, Vector3},
};
use std::fmt;
use std::hash::Hash;

use crate::math::vector2::Vector2;
use num_traits::Euclid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An iterator that yields all `BlockPos` positions within a cuboid region.
pub struct BlockPosIterator {
    /// The minimum X coordinate (inclusive).
    start_x: i32,
    /// The minimum Y coordinate (inclusive).
    start_y: i32,
    /// The minimum Z coordinate (inclusive).
    start_z: i32,
    /// The maximum X coordinate (inclusive).
    end_x: i32,
    /// The maximum Y coordinate (inclusive).
    end_y: i32,
    /// The maximum Z coordinate (inclusive).
    end_z: i32,
    /// The current iteration index.
    index: usize,
    /// The total number of positions to iterate.
    count: usize,
}

impl BlockPosIterator {
    /// Creates a new `BlockPosIterator` over the specified inclusive cuboid region.
    ///
    /// # Arguments
    /// - `start_x` – The minimum X coordinate (inclusive).
    /// - `start_y` – The minimum Y coordinate (inclusive).
    /// - `start_z` – The minimum Z coordinate (inclusive).
    /// - `end_x` – The maximum X coordinate (inclusive).
    /// - `end_y` – The maximum Y coordinate (inclusive).
    /// - `end_z` – The maximum Z coordinate (inclusive).
    ///
    /// # Returns
    /// A new `BlockPosIterator` instance.
    #[must_use]
    pub const fn new(
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
        Self {
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

/// An iterator that yields `BlockPos` positions outward from a centre point.
pub struct OutwardIterator {
    /// The X coordinate of the center point.
    center_x: i32,
    /// The Y coordinate of the center point.
    center_y: i32,
    /// The Z coordinate of the center point.
    center_z: i32,
    /// The maximum absolute X difference from the center.
    range_x: i32,
    /// The maximum absolute Y difference from the center.
    range_y: i32,
    /// The maximum absolute Z difference from the center.
    range_z: i32,
    /// The maximum Manhattan distance to iterate.
    max_manhattan_distance: i32,
    /// The current position being considered.
    pos: BlockPos,
    /// The current Manhattan distance from the center.
    manhattan_distance: i32,
    /// The current X limit for this Manhattan distance.
    limit_x: i32,
    /// The current Y limit for this Manhattan distance.
    limit_y: i32,
    /// The current X offset.
    dx: i32,
    /// The current Y offset.
    dy: i32,
    /// Whether to swap Z sign for the next position.
    swap_z: bool,
}

impl OutwardIterator {
    /// Creates a new `OutwardIterator` that yields positions outward from a centre point.
    ///
    /// # Arguments
    /// - `center` – The central `BlockPos` from which to start iterating.
    /// - `range_x` – The maximum absolute difference allowed in the X-coordinate from the centre.
    /// - `range_y` – The maximum absolute difference allowed in the Y-coordinate from the centre.
    /// - `range_z` – The maximum absolute difference allowed in the Z-coordinate from the centre.
    ///
    /// # Returns
    /// A new `OutwardIterator` instance.
    #[must_use]
    pub const fn new(center: BlockPos, range_x: i32, range_y: i32, range_z: i32) -> Self {
        let max_manhattan_distance = range_x + range_y + range_z;
        Self {
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

/// Represents a position in a 3D block grid.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockPos(pub Vector3<i32>);

impl BlockPos {
    /// The zero position (0, 0, 0).
    pub const ZERO: Self = Self::new(0, 0, 0);

    /// Creates a new `BlockPos` with the given coordinates.
    ///
    /// # Arguments
    /// - `x` – The X coordinate.
    /// - `y` – The Y coordinate.
    /// - `z` – The Z coordinate.
    ///
    /// # Returns
    /// A new `BlockPos` instance.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(Vector3::new(x, y, z))
    }

    /// Iterates through all `BlockPos` within a cuboid region defined by two corner points.
    ///
    /// # Arguments
    /// - `start` – One corner of the cuboid region.
    /// - `end` – The opposite corner of the cuboid region.
    ///
    /// # Returns
    /// A `BlockPosIterator` that yields each `BlockPos` within the defined cuboid.
    #[must_use]
    pub fn iterate(start: Self, end: Self) -> BlockPosIterator {
        BlockPosIterator::new(
            start.0.x.min(end.0.x),
            start.0.y.min(end.0.y),
            start.0.z.min(end.0.z),
            start.0.x.max(end.0.x),
            start.0.y.max(end.0.y),
            start.0.z.max(end.0.z),
        )
    }

    /// Iterates through `BlockPos` objects outward from a specified `center` point.
    ///
    /// # Arguments
    /// - `center` – The central `BlockPos` from which to start iterating.
    /// - `range_x` – The maximum absolute difference allowed in the X-coordinate from the centre.
    /// - `range_y` – The maximum absolute difference allowed in the Y-coordinate from the centre.
    /// - `range_z` – The maximum absolute difference allowed in the Z-coordinate from the centre.
    ///
    /// # Returns
    /// An `OutwardIterator` that yields `BlockPos` instances in the described outward order.
    #[must_use]
    pub const fn iterate_outwards(
        center: Self,
        range_x: i32,
        range_y: i32,
        range_z: i32,
    ) -> OutwardIterator {
        OutwardIterator::new(center, range_x, range_y, range_z)
    }

    /// Creates a block position iterator over the specified inclusive range.
    ///
    /// # Arguments
    /// - `start_x` – The minimum X coordinate (inclusive).
    /// - `start_y` – The minimum Y coordinate (inclusive).
    /// - `start_z` – The minimum Z coordinate (inclusive).
    /// - `end_x` – The maximum X coordinate (inclusive).
    /// - `end_y` – The maximum Y coordinate (inclusive).
    /// - `end_z` – The maximum Z coordinate (inclusive).
    ///
    /// # Returns
    /// A `BlockPosIterator` that yields each `BlockPos` within the defined range.
    #[must_use]
    pub const fn iterate_block_pos(
        start_x: i32,
        start_y: i32,
        start_z: i32,
        end_x: i32,
        end_y: i32,
        end_z: i32,
    ) -> BlockPosIterator {
        BlockPosIterator::new(start_x, start_y, start_z, end_x, end_y, end_z)
    }

    /// Returns both the chunk position and the relative position within that chunk.
    ///
    /// # Returns
    /// A tuple containing:
    /// - The chunk position as a `Vector2<i32>`.
    /// - The relative position within the chunk as a `Vector3<i32>`.
    #[must_use]
    pub const fn chunk_and_chunk_relative_position(&self) -> (Vector2<i32>, Vector3<i32>) {
        (self.chunk_position(), self.chunk_relative_position())
    }

    /// Returns the chunk position containing this block.
    ///
    /// # Returns
    /// A `Vector2<i32>` representing the chunk coordinates (X and Z only).
    #[must_use]
    pub const fn chunk_position(&self) -> Vector2<i32> {
        let z_chunk = self.0.z.div_euclid(16);
        let x_chunk = self.0.x.div_euclid(16);
        Vector2 {
            x: x_chunk,
            y: z_chunk,
        }
    }

    /// Returns the position of this block relative to its containing chunk.
    ///
    /// # Returns
    /// A `Vector3<i32>` with coordinates in the range [0, 15] for X and Z,
    /// and the actual Y coordinate (since chunks extend infinitely in Y).
    #[must_use]
    pub const fn chunk_relative_position(&self) -> Vector3<i32> {
        let z_chunk = self.0.z.rem_euclid(16);
        let x_chunk = self.0.x.rem_euclid(16);
        Vector3 {
            x: x_chunk,
            y: self.0.y,
            z: z_chunk,
        }
    }

    /// Returns the position of this block relative to its containing section.
    ///
    /// # Returns
    /// A `Vector3<i32>` with all coordinates in the range [0, 15].
    #[must_use]
    pub fn section_relative_position(&self) -> Vector3<i32> {
        let (_z_chunk, z_rem) = self.0.z.div_rem_euclid(&16);
        let (_x_chunk, x_rem) = self.0.x.div_rem_euclid(&16);
        let (_y_chunk, y_rem) = self.0.y.div_rem_euclid(&16);

        // NOTE: Since we divide by 16 remnant can never exceed u8
        Vector3 {
            x: x_rem,
            z: z_rem,
            y: y_rem,
        }
    }

    /// Creates a `BlockPos` from a packed 64-bit integer representation.
    ///
    /// The packing format is:
    /// - Bits 38-63: X coordinate (26 bits)
    /// - Bits 12-37: Z coordinate (26 bits)
    /// - Bits 0-11: Y coordinate (12 bits)
    ///
    /// # Arguments
    /// - `encoded_position` – The packed 64-bit integer.
    ///
    /// # Returns
    /// The decoded `BlockPos`.
    #[must_use]
    pub const fn from_i64(encoded_position: i64) -> Self {
        Self(Vector3 {
            x: (encoded_position >> 38) as i32,
            y: (encoded_position << 52 >> 52) as i32,
            z: (encoded_position << 26 >> 38) as i32,
        })
    }

    /// Creates a `BlockPos` by flooring the given floating-point coordinates.
    ///
    /// # Arguments
    /// - `x` – The X coordinate as a float.
    /// - `y` – The Y coordinate as a float.
    /// - `z` – The Z coordinate as a float.
    ///
    /// # Returns
    /// A `BlockPos` with each coordinate floored to the nearest integer.
    #[must_use]
    pub fn floored(x: f64, y: f64, z: f64) -> Self {
        Self(Vector3::new(
            x.floor() as i32,
            y.floor() as i32,
            z.floor() as i32,
        ))
    }

    /// Creates a `BlockPos` by flooring the given vector.
    ///
    /// # Arguments
    /// - `pos` – The `Vector3<f64>` to floor.
    ///
    /// # Returns
    /// A `BlockPos` with each component floored to the nearest integer.
    #[must_use]
    pub fn floored_v(pos: Vector3<f64>) -> Self {
        Self(Vector3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        ))
    }

    /// Creates a `BlockPos` by ceiling the given vector.
    ///
    /// # Arguments
    /// - `pos` – The `Vector3<f64>` to ceiling.
    ///
    /// # Returns
    /// A `BlockPos` with each component ceiled to the nearest integer.
    #[must_use]
    pub fn ceiled_v(pos: Vector3<f64>) -> Self {
        Self(Vector3::new(
            pos.x.ceil() as i32,
            pos.y.ceil() as i32,
            pos.z.ceil() as i32,
        ))
    }

    /// Converts this block position to a `Vector3<f64>` with 0.5 added to X and Z.
    ///
    /// This centers the position on the block's X and Z axes while keeping
    /// the exact Y coordinate.
    ///
    /// # Returns
    /// A `Vector3<f64>` representing the centered block position.
    #[must_use]
    pub fn to_f64(&self) -> Vector3<f64> {
        Vector3::new(
            f64::from(self.0.x) + 0.5,
            f64::from(self.0.y),
            f64::from(self.0.z) + 0.5,
        )
    }

    /// Converts this block position to a `Vector3<f64>` with 0.5 added to all axes.
    ///
    /// # Returns
    /// A `Vector3<f64>` representing the fully centered block position.
    #[must_use]
    pub fn to_centered_f64(&self) -> Vector3<f64> {
        Vector3::new(
            f64::from(self.0.x) + 0.5,
            f64::from(self.0.y) + 0.5,
            f64::from(self.0.z) + 0.5,
        )
    }

    /// Adds a vector offset to this block position.
    ///
    /// # Arguments
    /// - `offset` – The `Vector3<i32>` offset to add.
    ///
    /// # Returns
    /// A new `BlockPos` at the offset position.
    #[must_use]
    pub fn offset(&self, offset: Vector3<i32>) -> Self {
        Self(self.0 + offset)
    }

    /// Adds raw coordinate offsets to this block position.
    ///
    /// # Arguments
    /// - `x` – The X offset.
    /// - `y` – The Y offset.
    /// - `z` – The Z offset.
    ///
    /// # Returns
    /// A new `BlockPos` at the offset position.
    #[must_use]
    pub const fn add(&self, x: i32, y: i32, z: i32) -> Self {
        Self::new(self.0.x + x, self.0.y + y, self.0.z + z)
    }

    /// Adds a directional offset multiplied by a factor.
    ///
    /// # Arguments
    /// - `offset` – The base offset vector.
    /// - `direction` – The multiplier to apply to the offset.
    ///
    /// # Returns
    /// A new `BlockPos` at the offset position.
    #[must_use]
    pub const fn offset_dir(&self, offset: Vector3<i32>, direction: i32) -> Self {
        Self(Vector3::new(
            self.0.x + offset.x * direction,
            self.0.y + offset.y * direction,
            self.0.z + offset.z * direction,
        ))
    }

    /// Returns the block position one block above (Y+1).
    ///
    /// # Returns
    /// A new `BlockPos` at (x, y+1, z).
    #[must_use]
    pub fn up(&self) -> Self {
        self.offset(Vector3::new(0, 1, 0))
    }

    /// Returns the block position several blocks above.
    ///
    /// # Arguments
    /// - `height` – The number of blocks to move up.
    ///
    /// # Returns
    /// A new `BlockPos` at (x, y+height, z).
    #[must_use]
    pub fn up_height(&self, height: i32) -> Self {
        self.offset(Vector3::new(0, height, 0))
    }

    /// Returns the block position one block below (Y-1).
    ///
    /// # Returns
    /// A new `BlockPos` at (x, y-1, z).
    #[must_use]
    pub fn down(&self) -> Self {
        self.offset(Vector3::new(0, -1, 0))
    }

    /// Returns the block position several blocks below.
    ///
    /// # Arguments
    /// - `height` – The number of blocks to move down.
    ///
    /// # Returns
    /// A new `BlockPos` at (x, y-height, z).
    #[must_use]
    pub fn down_height(&self, height: i32) -> Self {
        self.offset(Vector3::new(0, -height, 0))
    }

    /// Returns the block position one block west (X-1).
    ///
    /// # Returns
    /// A new `BlockPos` at (x-1, y, z).
    #[must_use]
    pub fn west(&self) -> Self {
        self.offset(Vector3::new(-1, 0, 0))
    }

    /// Returns the block position one block north (Z-1).
    ///
    /// # Returns
    /// A new `BlockPos` at (x, y, z-1).
    #[must_use]
    pub fn north(&self) -> Self {
        self.offset(Vector3::new(0, 0, -1))
    }

    /// Returns the block position one block east (X+1).
    ///
    /// # Returns
    /// A new `BlockPos` at (x+1, y, z).
    #[must_use]
    pub fn east(&self) -> Self {
        self.offset(Vector3::new(1, 0, 0))
    }

    /// Returns the block position one block south (Z+1).
    ///
    /// # Returns
    /// A new `BlockPos` at (x, y, z+1).
    #[must_use]
    pub fn south(&self) -> Self {
        self.offset(Vector3::new(0, 0, 1))
    }

    /// Calculates the Manhattan distance between this block position and another.
    ///
    /// # Arguments
    /// - `other` – The other block position.
    ///
    /// # Returns
    /// The Manhattan distance (sum of absolute differences in each axis).
    #[must_use]
    pub const fn manhattan_distance(&self, other: Self) -> i32 {
        let x = (other.0.x - self.0.x).abs();
        let y = (other.0.y - self.0.y).abs();
        let z = (other.0.z - self.0.z).abs();
        x + y + z
    }

    /// Calculates the squared Euclidean distance between this block position and another.
    ///
    /// # Arguments
    /// - `other` – The other block position.
    ///
    /// # Returns
    /// The squared Euclidean distance.
    #[must_use]
    pub fn squared_distance(&self, other: &Self) -> i32 {
        self.0.squared_distance_to_vec(&other.0)
    }

    /// Packs this block position into a 64-bit integer.
    ///
    /// The packing format is:
    /// - Bits 38-63: X coordinate (26 bits)
    /// - Bits 12-37: Z coordinate (26 bits)
    /// - Bits 0-11: Y coordinate (12 bits)
    ///
    /// # Returns
    /// A packed 64-bit integer representing this block position.
    #[must_use]
    pub fn as_long(&self) -> i64 {
        ((i64::from(self.0.x) & 0x03FF_FFFF) << 38)
            | ((i64::from(self.0.z) & 0x03FF_FFFF) << 12)
            | (i64::from(self.0.y) & 0xFFF)
    }
}

impl Serialize for BlockPos {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(self.as_long())
    }
}

impl<'de> Deserialize<'de> for BlockPos {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = BlockPos;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("An i64 int")
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
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

/// Returns the chunk section position containing the given block position.
///
/// # Arguments
/// - `block_pos` – The block position.
///
/// # Returns
/// A `Vector3<i32>` representing the chunk section coordinates.
#[must_use]
pub const fn chunk_section_from_pos(block_pos: &BlockPos) -> Vector3<i32> {
    let block_pos = block_pos.0;
    Vector3::new(
        get_section_cord(block_pos.x),
        get_section_cord(block_pos.y),
        get_section_cord(block_pos.z),
    )
}

/// Gets the local coordinate within a chunk section (0-15).
///
/// # Arguments
/// - `cord` – The global coordinate.
///
/// # Returns
/// The local coordinate (bits 0-3) in the range [0, 15].
#[must_use]
pub const fn get_local_cord(cord: i32) -> i32 {
    cord & 15
}

/// Packs local coordinates within a chunk section into a 16-bit integer.
///
/// The packing format matches `vector3::packed_local`:
/// - Bits 8-15: X coordinate (4 bits)
/// - Bits 4-7: Z coordinate (4 bits)
/// - Bits 0-3: Y coordinate (4 bits)
///
/// # Arguments
/// - `block_pos` – The block position.
///
/// # Returns
/// A packed 16-bit integer containing the local coordinates.
#[must_use]
pub const fn pack_local_chunk_section(block_pos: &BlockPos) -> i16 {
    let x = get_local_cord(block_pos.0.x);
    let z = get_local_cord(block_pos.0.z);
    let y = get_local_cord(block_pos.0.y);
    vector3::packed_local(&Vector3::new(x, y, z))
}
