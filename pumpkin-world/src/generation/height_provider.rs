use pumpkin_util::random::RandomGenerator;
use serde::Deserialize;

use super::y_offset::YOffset;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum HeightProvider {
    Uniform(UniformHeightProvider),
}

impl HeightProvider {
    pub fn get(&self, random: &mut RandomGenerator, min_y: i8, height: u16) -> i32 {
        match self {
            HeightProvider::Uniform(uniform) => uniform.get(random, min_y, height),
        }
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

        random.next_bounded_i32(max - min + 1) + min
    }
}
