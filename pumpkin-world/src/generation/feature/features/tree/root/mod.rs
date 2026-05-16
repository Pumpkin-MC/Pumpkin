use mangrove::MangroveRootPlacer;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub mod mangrove;

pub enum RootPlacer {
    Mangrove(MangroveRootPlacer),
}

impl RootPlacer {
    pub fn trunk_offset(&self, pos: BlockPos, random: &mut RandomGenerator) -> BlockPos {
        match self {
            Self::Mangrove(p) => p.trunk_offset(pos, random),
        }
    }

    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        trunk_pos: BlockPos,
    ) -> Option<Vec<BlockPos>> {
        match self {
            Self::Mangrove(p) => p.generate(chunk, random, pos, trunk_pos),
        }
    }
}
