use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::placed_features::PlacedFeature};

#[derive(Deserialize)]
pub struct SimpleRandomFeature {
    features: Vec<PlacedFeature>,
}

impl SimpleRandomFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let i = random.next_bounded_i32(self.features.len() as i32);
        let feature = &self.features[i as usize];
        feature.generate(chunk, min_y, height, feature_name, random, pos)
    }
}
