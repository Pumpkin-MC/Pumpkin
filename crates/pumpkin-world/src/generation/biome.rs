use std::cell::RefCell;

use pumpkin_util::math::{square_f64, vector3::Vector3};

use super::biome_coords;

// This blends biome boundaries, returning which biome to populate the surface on based on the seed
pub fn get_biome_blend(
    bottom_y: i8,
    height: u16,
    seed: i64,
    x: i32,
    y: i32,
    z: i32,
) -> Vector3<i32> {
    // This is the "left" side of the biome boundary
    let offset_x = x - 2;
    let offset_y = y - 2;
    let offset_z = z - 2;
    let biome_x = biome_coords::from_block(offset_x);
    let biome_y = biome_coords::from_block(offset_y);
    let biome_z = biome_coords::from_block(offset_z);
    // &'ing 3 gives values of 0-3, it is also the data we removed when converting to biome coords
    // This is effectively "quarters" into the biome
    // Original was "/ 4.0" but we use "* 0.25" as multiplication can be faster
    let quarters_x = (offset_x & 0b11) as f64 * 0.25;
    let quarters_y = (offset_y & 0b11) as f64 * 0.25;
    let quarters_z = (offset_z & 0b11) as f64 * 0.25;

    let offsets = cell_offsets(seed, biome_x, biome_y, biome_z);
    let mut best_permutation = 0;
    let mut best_score = f64::INFINITY;
    for (permutation, offset) in offsets.iter().enumerate() {
        let shifted_quarters_x = if (permutation & 0b100) == 0 {
            quarters_x
        } else {
            quarters_x - 1.0
        };
        let shifted_quarters_y = if (permutation & 0b010) == 0 {
            quarters_y
        } else {
            quarters_y - 1.0
        };
        let shifted_quarters_z = if (permutation & 0b001) == 0 {
            quarters_z
        } else {
            quarters_z - 1.0
        };

        let permutation_score = score_offset(
            offset,
            shifted_quarters_x,
            shifted_quarters_y,
            shifted_quarters_z,
        );

        if best_score > permutation_score {
            best_score = permutation_score;
            best_permutation = permutation;
        }
    }

    // Now check if we want to use the "left" side or the "right" side
    let biome_x = if (best_permutation & 0b100) == 0 {
        biome_x
    } else {
        biome_x + 1
    };
    let biome_y = if (best_permutation & 0b010) == 0 {
        biome_y
    } else {
        biome_y + 1
    };
    let biome_z = if (best_permutation & 0b001) == 0 {
        biome_z
    } else {
        biome_z + 1
    };

    // Java's `getBiomeForNoiseGen`
    let bottom_y = bottom_y as i32;
    let biome_bottom = biome_coords::from_block(bottom_y);
    let biome_top = biome_bottom + biome_coords::from_block(height as i32) - 1;
    let biome_y = biome_y.clamp(biome_bottom, biome_top);

    Vector3::new(biome_x, biome_y, biome_z)
}

type CellOffsets = [[f64; 3]; 8];
type CellKey = (i64, i32, i32, i32);

thread_local! {
    static LAST_CELL_OFFSETS: RefCell<Option<(CellKey, CellOffsets)>> =
        const { RefCell::new(None) };
}

// A column scan asks for the same biome cell four times in a row, so the corner offsets are reused.
fn cell_offsets(seed: i64, x: i32, y: i32, z: i32) -> CellOffsets {
    let key = (seed, x, y, z);
    LAST_CELL_OFFSETS.with_borrow_mut(|last| {
        if let Some((last_key, offsets)) = last
            && *last_key == key
        {
            return *offsets;
        }
        let mut offsets = [[0.0; 3]; 8];
        for (permutation, offset) in offsets.iter_mut().enumerate() {
            let shifted_x = if (permutation & 0b100) == 0 { x } else { x + 1 };
            let shifted_y = if (permutation & 0b010) == 0 { y } else { y + 1 };
            let shifted_z = if (permutation & 0b001) == 0 { z } else { z + 1 };
            *offset = corner_offset(seed, shifted_x, shifted_y, shifted_z);
        }
        *last = Some((key, offsets));
        offsets
    })
}

// A random offset (+/- 0.0-0.8ish) for one corner of a biome cell
const fn corner_offset(seed: i64, x: i32, y: i32, z: i32) -> [f64; 3] {
    let mix = salt_mix(seed, x as i64);
    let mix = salt_mix(mix, y as i64);
    let mix = salt_mix(mix, z as i64);
    let mix = salt_mix(mix, x as i64);
    let mix = salt_mix(mix, y as i64);
    let mix = salt_mix(mix, z as i64);
    let offset_x = scale_mix(mix);
    let mix = salt_mix(mix, seed);
    let offset_y = scale_mix(mix);
    let mix = salt_mix(mix, seed);
    let offset_z = scale_mix(mix);
    [offset_x, offset_y, offset_z]
}

// The hypotenuse squared of the quarters plus the corner's offset
const fn score_offset(offset: &[f64; 3], x_part: f64, y_part: f64, z_part: f64) -> f64 {
    square_f64(z_part + offset[2]) + square_f64(y_part + offset[1]) + square_f64(x_part + offset[0])
}

#[cfg(test)]
const fn score_permutation(
    seed: i64,
    x: i32,
    y: i32,
    z: i32,
    x_part: f64,
    y_part: f64,
    z_part: f64,
) -> f64 {
    score_offset(&corner_offset(seed, x, y, z), x_part, y_part, z_part)
}

#[inline]
pub const fn scale_mix(l: i64) -> f64 {
    // Shifting and then masking with 1023 (1024 - 1)
    // This is mathematically identical to floor_mod(l >> 24, 1024)
    // but executes in a single CPU cycle.
    let d = ((l >> 24) & 1023) as f64 / 1024.0;

    (d - 0.5) * 0.9
}

#[inline]
const fn salt_mix(seed: i64, salt: i64) -> i64 {
    let mixed_seed = seed.wrapping_mul(
        seed.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407),
    );
    mixed_seed.wrapping_add(salt)
}

#[cfg(test)]
mod test {
    use pumpkin_util::{math::vector3::Vector3, read_data_from_file};

    use crate::{
        biome::hash_seed,
        generation::biome::{get_biome_blend, scale_mix, score_permutation},
    };

    use super::salt_mix;

    #[test]
    fn mix_seed() {
        let seed = salt_mix(12345678, 12345678);
        assert_eq!(seed, 2937271135939595220);
    }

    #[test]
    fn permutation() {
        let seed = hash_seed(0);
        let score = score_permutation(seed, 123, 456, 456, 0.25, 0.5, 0.75);
        assert_eq!(score, 1.276986312866211);
    }

    #[test]
    fn biome_blend() {
        let biome_pos = get_biome_blend(-64, 384, 1234567890, 123, 123, 123);
        assert_eq!(biome_pos, Vector3::new(31, 30, 30));
    }

    #[test]
    fn scale() {
        let seed = scale_mix(12345678);
        assert_eq!(seed, -0.45);
    }

    #[test]
    fn chunk_wide_blend() {
        let data: Vec<(i32, i32, i32, i32, i32, i32)> =
            read_data_from_file!("../../../../assets/tests/biome_mixer.json");

        let seed = hash_seed((-777i64) as u64);
        for (i, (x, y, z, result_x, result_y, result_z)) in data.into_iter().enumerate() {
            let result = get_biome_blend(i8::MIN, u16::MAX, seed, x, y, z);
            let expected = Vector3::new(result_x, result_y, result_z);
            assert_eq!(
                result, expected,
                "Expected: {expected:?}, was: {result:?} ({i})"
            );
        }
    }
}
