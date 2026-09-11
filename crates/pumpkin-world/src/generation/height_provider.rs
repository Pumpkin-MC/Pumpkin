use std::num::NonZero;

use pumpkin_util::{
    random::{RandomGenerator, RandomImpl},
    y_offset::YOffset,
};
use tracing::warn;

pub enum HeightProvider {
    Uniform(UniformHeightProvider),
    Trapezoid(TrapezoidHeightProvider),
    VeryBiasedToBottom(VeryBiasedToBottomHeightProvider),
}

impl HeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        match self {
            Self::Uniform(provider) => provider.get(random, min_y, height),
            Self::Trapezoid(provider) => provider.get(random, min_y, height),
            Self::VeryBiasedToBottom(provider) => provider.get(random, min_y, height),
        }
    }
}

pub struct VeryBiasedToBottomHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub inner: Option<NonZero<u32>>,
}

impl VeryBiasedToBottomHeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y as i16, height);
        let max = self.max_inclusive.get_y(min_y as i16, height);
        let inner = self.inner.map_or(1, std::num::NonZero::get) as i32;

        if max - min - inner < 0 {
            warn!("Empty height range: min={min}, max={max}, inner={inner}");
            return min;
        }

        let upper_inclusive = random.next_inbetween_i32(min + inner, max);
        let biased_upper_inclusive = random.next_inbetween_i32(min, upper_inclusive - 1);
        random.next_inbetween_i32(min, biased_upper_inclusive - 1 + inner)
    }
}

pub struct UniformHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
}

impl UniformHeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y as i16, height);
        let max = self.max_inclusive.get_y(min_y as i16, height);

        if min > max {
            warn!("Empty height range: min={min}, max={max}");
            return min;
        }

        random.next_inbetween_i32(min, max)
    }
}

pub struct TrapezoidHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub plateau: Option<i32>,
}

impl TrapezoidHeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let plateau = self.plateau.unwrap_or(0);
        let i = self.min_inclusive.get_y(min_y as i16, height);
        let j = self.max_inclusive.get_y(min_y as i16, height);

        if i > j {
            warn!("Empty height range");
            return i;
        }

        let k = j - i;
        if plateau >= k {
            return random.next_inbetween_i32(i, j);
        }

        let l = (k - plateau) / 2;
        let m = k - l;
        i + random.next_inbetween_i32(0, m) + random.next_inbetween_i32(0, l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::random::xoroshiro128::Xoroshiro;
    use pumpkin_util::y_offset::{AboveBottom, BelowTop};

    #[test]
    fn very_biased_to_bottom_parity() {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(12345));
        let provider = VeryBiasedToBottomHeightProvider {
            min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10 }),
            max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10 }),
            inner: NonZero::new(8),
        };

        // Nether height 128, min_y 0 -> min = 10, max = 118
        for _ in 0..1000 {
            let y = provider.get(&mut random, 0, 128);
            assert!((10..=118).contains(&y), "y out of bounds: {y}");
        }

        // Empty height range
        let empty_provider = VeryBiasedToBottomHeightProvider {
            min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 100 }),
            max_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10 }),
            inner: NonZero::new(8),
        };
        assert_eq!(empty_provider.get(&mut random, 0, 128), 100);
    }
}
