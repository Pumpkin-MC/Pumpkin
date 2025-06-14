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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// Aka Block Position
pub struct BlockPos(pub Vector3<i32>);

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(Vector3::new(x, y, z))
    }

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

    pub fn floored(x: f64, y: f64, z: f64) -> Self {
        Self(Vector3::new(
            x.floor() as i32,
            y.floor() as i32,
            z.floor() as i32,
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

    pub fn iterate_recursively<F>(
        start: BlockPos,
        end: BlockPos,
        mut visitor: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(BlockPos) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>,
    {
        fn recurse_visit<F>(
            current: BlockPos,
            start: BlockPos,
            end: BlockPos,
            visitor: &mut F,
            visited: &mut std::collections::HashSet<BlockPos>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where
            F: FnMut(BlockPos) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>,
        {
            if visited.contains(&current) {
                return Ok(());
            }

            visited.insert(current); // Visit current position
            let should_continue = visitor(current)?;
            if !should_continue {
                return Ok(());
            }

            // Calculate bounds
            let min_x = start.0.x.min(end.0.x);
            let max_x = start.0.x.max(end.0.x);
            let min_y = start.0.y.min(end.0.y);
            let max_y = start.0.y.max(end.0.y);
            let min_z = start.0.z.min(end.0.z);
            let max_z = start.0.z.max(end.0.z);

            // Recursively visit adjacent positions within bounds
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }

                        let next =
                            BlockPos::new(current.0.x + dx, current.0.y + dy, current.0.z + dz);

                        // Check bounds
                        if next.0.x >= min_x
                            && next.0.x <= max_x
                            && next.0.y >= min_y
                            && next.0.y <= max_y
                            && next.0.z >= min_z
                            && next.0.z <= max_z
                        {
                            recurse_visit(next, start, end, visitor, visited)?;
                        }
                    }
                }
            }

            Ok(())
        }

        let mut visited = std::collections::HashSet::new();
        recurse_visit(start, start, end, &mut visitor, &mut visited)
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
