use mangrove::MangroveRootPlacer;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};

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
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
        trunk_pos: BlockPos,
    ) -> Option<Vec<BlockPos>> {
        match self {
            Self::Mangrove(p) => p.generate(chunk, block_registry, random, pos, trunk_pos),
        }
    }
}
