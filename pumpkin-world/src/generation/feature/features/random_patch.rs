use std::sync::Arc;

use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::placed_features::PlacedFeature,
    world::{BlockRegistryExt, SimpleWorld},
};

#[derive(Deserialize)]
pub struct RandomPatchFeature {
    tries: u8,
    xz_spread: u8,
    y_spread: u8,
    feature: Box<PlacedFeature>,
}

impl RandomPatchFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        return false;
        let mut i = 0;
        let xz = self.xz_spread as i32 + 1;
        let y = self.y_spread as i32 + 1;
        for _ in 0..self.tries {
            dbg!(pos.0.y);
            let pos = Vector3::new(
                pos.0.x + (random.next_bounded_i32(xz) - random.next_bounded_i32(xz)),
                pos.0.y + (random.next_bounded_i32(y) - random.next_bounded_i32(y)),
                pos.0.z + (random.next_bounded_i32(xz) - random.next_bounded_i32(xz)),
            );
            dbg!(pos.y);
            if !self.feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature,
                random,
                BlockPos(pos),
            ) {
                continue;
            }
            i += 1;
        }
        i > 0
    }
}
