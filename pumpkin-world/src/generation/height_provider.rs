use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use super::y_offset::YOffset;

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
            HeightProvider::Uniform(provider) => provider.get(random, min_y, height),
            HeightProvider::Trapezoid(provider) => provider.get(random, min_y, height),
            HeightProvider::VeryBiasedToBottom(provider) => provider.get(random, min_y, height),
        }
    }
}

#[derive(Deserialize)]
pub struct VeryBiasedToBottomHeightProvider {
    min_inclusive: YOffset,
    max_inclusive: YOffset,
    inner: Option<u32>,
}

impl VeryBiasedToBottomHeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        let min = self.min_inclusive.get_y(min_y, height) as i32;
        let max = self.max_inclusive.get_y(min_y, height) as i32;
        let inner = self.inner.unwrap_or(1) as i32;

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
        let min = self.min_inclusive.get_y(min_y, height) as i32;
        let max = self.max_inclusive.get_y(min_y, height) as i32;

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
        let i = self.min_inclusive.get_y(min_y, height);
        let j = self.max_inclusive.get_y(min_y, height);

        if i > j {
            log::warn!("Empty height range");
            return i as i32;
        }

        let k = j - i;
        if plateau >= k as i32 {
            return random.next_inbetween_i32(i as i32, j as i32);
        }

        let l = (k as i32 - plateau) / 2;
        let m = k as i32 - l;
        i as i32 + random.next_inbetween_i32(0, m) + random.next_inbetween_i32(0, l)
    }
}
