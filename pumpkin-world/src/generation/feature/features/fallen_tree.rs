use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, generation::block_state_provider::BlockStateProvider};

#[derive(Deserialize)]
pub struct FallenTreeFeature {
    trunk_provider: BlockStateProvider,
}

impl FallenTreeFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        height: u16,
        feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        false
    }

    fn gen_stump(&self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, pos: BlockPos) {
        chunk.set_block_state(&pos.0, &self.trunk_provider.get(random, pos));
    }
}
