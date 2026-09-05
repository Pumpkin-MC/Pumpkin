use crate::population_seed_fn;

use super::{RandomDeriver, RandomImpl, gaussian::GaussianGenerator, xoroshiro128::Xoroshiro};

/// A Xoroshiro source accessed through Java `WorldgenRandom` bit-source semantics.
///
/// `WorldgenRandom::next(bits)` takes the high bits of a complete underlying
/// `nextLong()` draw. This differs from direct `XoroshiroRandomSource` methods.
pub struct WorldgenRandom {
    source: Xoroshiro,
    internal_next_gaussian: Option<f64>,
}

impl WorldgenRandom {
    population_seed_fn!();

    #[must_use]
    pub const fn from_seed(seed: u64) -> Self {
        Self {
            source: Xoroshiro::from_seed(seed),
            internal_next_gaussian: None,
        }
    }

    const fn next(&mut self, bits: u64) -> u64 {
        self.source.next(bits)
    }
}

impl GaussianGenerator for WorldgenRandom {
    fn stored_next_gaussian(&self) -> Option<f64> {
        self.internal_next_gaussian
    }

    fn set_stored_next_gaussian(&mut self, value: Option<f64>) {
        self.internal_next_gaussian = value;
    }
}

impl RandomImpl for WorldgenRandom {
    fn split(&mut self) -> Self {
        Self {
            source: self.source.split(),
            internal_next_gaussian: None,
        }
    }

    fn next_splitter(&mut self) -> RandomDeriver {
        RandomDeriver::Xoroshiro(self.source.next_splitter())
    }

    fn next_i32(&mut self) -> i32 {
        self.next(32) as i32
    }

    fn next_bounded_i32(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if bound & (bound - 1) == 0 {
            return ((i64::from(bound) * self.next(31) as i64) >> 31) as i32;
        }

        loop {
            let value = self.next(31) as i32;
            let result = value % bound;
            if value.wrapping_sub(result).wrapping_add(bound - 1) >= 0 {
                return result;
            }
        }
    }

    fn next_i64(&mut self) -> i64 {
        let high = self.next_i32();
        let low = self.next_i32();
        (i64::from(high) << 32).wrapping_add(i64::from(low))
    }

    fn next_bool(&mut self) -> bool {
        self.next(1) != 0
    }

    fn next_f32(&mut self) -> f32 {
        self.next(24) as f32 * 5.960_464_5E-8f32
    }

    fn next_f64(&mut self) -> f64 {
        let high = self.next(26);
        let low = self.next(27);
        ((high << 27) + low) as f64 * 1.110_223_024_625_156_5E-16
    }

    fn next_gaussian(&mut self) -> f64 {
        self.calculate_gaussian()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::get_decorator_seed;

    const SEED: u64 = 1_786_192_857_164_469_025;

    #[test]
    fn decoration_seed_matches_vanilla() {
        let chunk_86 = WorldgenRandom::get_population_seed(SEED, 86 << 4, 600 << 4);
        let chunk_87 = WorldgenRandom::get_population_seed(SEED, 87 << 4, 600 << 4);
        assert_eq!(chunk_86, 0x36e9_895d_da2b_ad81);
        assert_eq!(chunk_87, 0xcdc9_553d_e86a_6171);
        assert_eq!(get_decorator_seed(chunk_86, 69, 9), 0x36e9_895d_da2d_0d56);
        assert_eq!(get_decorator_seed(chunk_87, 69, 9), 0xcdc9_553d_e86b_c146);
    }

    #[test]
    fn dry_grass_modifier_draws_match_vanilla() {
        let mut chunk_86 = WorldgenRandom::from_seed(0x36e9_895d_da2d_0d56);
        assert_eq!(chunk_86.next_f32().to_bits(), 0x3db2_ba18);
        assert_eq!(chunk_86.next_bounded_i32(16), 10);
        assert_eq!(chunk_86.next_bounded_i32(16), 6);
        assert_eq!(
            [
                chunk_86.next_bounded_i32(8),
                chunk_86.next_bounded_i32(8),
                chunk_86.next_bounded_i32(4),
                chunk_86.next_bounded_i32(4),
                chunk_86.next_bounded_i32(8),
                chunk_86.next_bounded_i32(8),
            ],
            [6, 5, 1, 3, 7, 7]
        );

        let mut chunk_87 = WorldgenRandom::from_seed(0xcdc9_553d_e86b_c146);
        assert_eq!(chunk_87.next_f32().to_bits(), 0x3f64_187a);
    }
}
