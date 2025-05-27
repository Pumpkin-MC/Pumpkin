use std::sync::Arc;

use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::placed_features::PlacedFeature,
    world::{BlockRegistryExt, SimpleWorld},
};

#[derive(Deserialize)]
pub struct SimpleRandomFeature {
    features: Vec<PlacedFeature>,
}

impl SimpleRandomFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let i = random.next_bounded_i32(self.features.len() as i32);
        let feature = &self.features[i as usize];
        feature.generate(
            chunk,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            pos,
        )
    }
}
