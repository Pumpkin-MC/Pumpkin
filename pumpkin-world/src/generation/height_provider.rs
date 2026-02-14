use std::num::NonZeroU32;

use pumpkin_util::{
    random::{RandomGenerator, RandomImpl},
    y_offset::YOffset,
};
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum HeightProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformHeightProvider),
    #[serde(rename = "minecraft:trapezoid")]
    Trapezoid(TrapezoidHeightProvider),
    #[serde(rename = "minecraft:very_biased_to_bottom")]
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

#[derive(Deserialize)]
pub struct VeryBiasedToBottomHeightProvider {
    min_inclusive: YOffset,
    max_inclusive: YOffset,
    inner: Option<NonZeroU32>,
}

impl VeryBiasedToBottomHeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y as i16, height);
        let max = self.max_inclusive.get_y(min_y as i16, height);
        let inner = self.inner.map_or(1, std::num::NonZero::get) as i32;

        let min_rnd = random.next_inbetween_i32(min + inner, max);
        let max_rnd = random.next_inbetween_i32(min, min_rnd - 1);

        random.next_inbetween_i32(min, max_rnd - 1 + inner)
    }
}

#[derive(Deserialize)]
pub struct UniformHeightProvider {
    min_inclusive: YOffset,
    max_inclusive: YOffset,
}

impl UniformHeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y as i16, height);
        let max = self.max_inclusive.get_y(min_y as i16, height);

        random.next_inbetween_i32(min, max)
    }
}

#[derive(Deserialize)]
pub struct TrapezoidHeightProvider {
    min_inclusive: YOffset,
    max_inclusive: YOffset,
    plateau: Option<i32>,
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
