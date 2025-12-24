use pumpkin_util::math::{floor_log2, smallest_encompassing_power_of_two};

pub mod block_pos {
    use pumpkin_util::math::vector3::Vector3;

    use super::{
        BIT_SHIFT_X, BIT_SHIFT_Z, BITS_X, BITS_Y, BITS_Z, SIZE_BITS_X, SIZE_BITS_Y, SIZE_BITS_Z,
    };

    #[inline]
    pub const fn unpack_x(packed: i64) -> i32 {
        ((packed << (64 - BIT_SHIFT_X - SIZE_BITS_X)) >> (64 - SIZE_BITS_X)) as i32
    }

    #[inline]
    pub const fn unpack_y(packed: i64) -> i32 {
        ((packed << (64 - SIZE_BITS_Y)) >> (64 - SIZE_BITS_Y)) as i32
    }

    #[inline]
    pub const fn unpack_z(packed: i64) -> i32 {
        ((packed << (64 - BIT_SHIFT_Z - SIZE_BITS_Z)) >> (64 - SIZE_BITS_Z)) as i32
    }

    #[inline]
    pub const fn packed(vec: &Vector3<i32>) -> i64 {
        let mut result = 0i64;
        // Need to go to i64 first to conserve sign
        result |= (vec.x as i64 & BITS_X as i64) << BIT_SHIFT_X;
        result |= (vec.z as i64 & BITS_Z as i64) << BIT_SHIFT_Z;
        result |= vec.y as i64 & BITS_Y as i64;
        result
    }
}

pub mod chunk_pos {
    use pumpkin_util::math::vector2::Vector2;

    use crate::generation::section_coords::get_offset_pos;

    // A chunk outside of normal bounds
    pub const MARKER: u64 = packed(1875066, 1875066);

    pub const fn packed(x: u64, y: u64) -> u64 {
        (x & 4294967295u64) | ((y & 4294967295u64) << 32)
    }

    pub const fn unpack_x(packed: u64) -> i32 {
        (packed & 4294967295u64) as i32
    }

    pub const fn unpack_z(packed: u64) -> i32 {
        ((packed >> 32) & 4294967295u64) as i32
    }

    pub fn get_offset_x(coord: i32, offset: i32) -> i32 {
        get_offset_pos(coord, offset)
    }

    pub fn get_center_x(coord: i32) -> i32 {
        get_offset_x(coord, 8)
    }

    pub fn get_center_z(coord: i32) -> i32 {
        get_offset_z(coord, 8)
    }

    pub fn get_offset_z(coord: i32, offset: i32) -> i32 {
        get_offset_pos(coord, offset)
    }

    pub const fn start_block_x(x: i32) -> i32 {
        x << 4
    }

    pub const fn end_block_x(x: i32) -> i32 {
        start_block_x(x) + 15
    }

    pub const fn start_block_z(z: i32) -> i32 {
        z << 4
    }

    pub const fn end_block_z(z: i32) -> i32 {
        start_block_z(z) + 15
    }

    pub const fn to_chunk_pos(vec: &Vector2<i32>) -> Vector2<i32> {
        Vector2::new(vec.x >> 4, vec.y >> 4)
    }
}

const MAX_BLOCK_AXIS: u32 = 30000000;
const SIZE_BITS_X: u8 = 1 + floor_log2(smallest_encompassing_power_of_two(MAX_BLOCK_AXIS));
const BITS_X: u64 = (1 << SIZE_BITS_X) - 1;
const SIZE_BITS_Z: u8 = SIZE_BITS_X;
const BITS_Z: u64 = (1 << SIZE_BITS_Z) - 1;
pub const SIZE_BITS_Y: u8 = 64 - SIZE_BITS_X - SIZE_BITS_Z;
const BITS_Y: u64 = (1 << SIZE_BITS_Y) - 1;
const BIT_SHIFT_Z: u8 = SIZE_BITS_Y;
const BIT_SHIFT_X: u8 = SIZE_BITS_Y + SIZE_BITS_Z;

pub const MAX_HEIGHT: u32 = (1 << SIZE_BITS_Y) - 32;
pub const MAX_COLUMN_HEIGHT: u32 = (MAX_HEIGHT >> 1) - 1;
pub const MIN_HEIGHT: i32 = MAX_COLUMN_HEIGHT as i32 - MAX_HEIGHT as i32 + 1;
pub const MIN_HEIGHT_CELL: i32 = MIN_HEIGHT << 4;

#[cfg(test)]
mod test {
    use pumpkin_util::math::vector3::Vector3;

    use super::{block_pos, chunk_pos};

    #[test]
    fn test_chunk_packing() {
        let x = 305135135_i32;
        let y = -1351513511_i32;
        let packed = chunk_pos::packed(x as u64, y as u64);
        assert_eq!(packed as i64, -5804706329542001121i64);
        assert_eq!(x, chunk_pos::unpack_x(packed));
        assert_eq!(y, chunk_pos::unpack_z(packed));
    }

    #[test]
    fn test_block_packing() {
        let pos = Vector3::new(-30000000, 120, 30000000);
        let packed = block_pos::packed(&pos);
        assert_eq!(packed, -8246337085439999880i64);
        assert_eq!(pos.x, block_pos::unpack_x(packed));
        assert_eq!(pos.y, block_pos::unpack_y(packed));
        assert_eq!(pos.z, block_pos::unpack_z(packed));

        for x in -10..=10 {
            for y in -10..=10 {
                for z in -10..=10 {
                    let pos = Vector3::new(x * 1000000, y * 10, z * 1000000);
                    let packed = block_pos::packed(&pos);
                    assert_eq!(pos.x, block_pos::unpack_x(packed));
                    assert_eq!(pos.y, block_pos::unpack_y(packed));
                    assert_eq!(pos.z, block_pos::unpack_z(packed));
                }
            }
        }
    }
}
