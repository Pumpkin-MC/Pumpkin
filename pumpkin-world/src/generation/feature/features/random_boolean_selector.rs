use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::placed_features::PlacedFeatureWrapper};

#[derive(Deserialize)]
pub struct RandomBooleanFeature {
    feature_true: Box<PlacedFeatureWrapper>,
    feature_false: Box<PlacedFeatureWrapper>,
}

impl RandomBooleanFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let val = random.next_bool();
        let feature = if val {
            &self.feature_true
        } else {
            &self.feature_false
        };
        feature
            .get()
            .generate(chunk, min_y, height, feature_name, random, pos)
    }
}
