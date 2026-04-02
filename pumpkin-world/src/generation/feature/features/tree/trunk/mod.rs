use fancy::FancyTrunkPlacer;
use pumpkin_data::BlockState;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use straight::StraightTrunkPlacer;

use super::{TreeFeature, TreeNode};
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::trunk::{
    bending::BendingTrunkPlacer, cherry::CherryTrunkPlacer, dark_oak::DarkOakTrunkPlacer,
    forking::ForkingTrunkPlacer, giant::GiantTrunkPlacer, mega_jungle::MegaJungleTrunkPlacer,
    upwards_branching::UpwardsBranchingTrunkPlacer,
};
use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

pub mod bending;
pub mod cherry;
pub mod dark_oak;
pub mod fancy;
pub mod forking;
pub mod giant;
pub mod mega_jungle;
pub mod straight;
pub mod upwards_branching;

pub struct TrunkPlacer {
    pub base_height: u8,
    pub height_rand_a: u8,
    pub height_rand_b: u8,
    pub r#type: TrunkType,
}

impl TrunkPlacer {
    pub fn get_height(&self, random: &mut RandomGenerator) -> u32 {
        self.base_height as u32
            + random.next_bounded_i32(self.height_rand_a as i32 + 1) as u32
            + random.next_bounded_i32(self.height_rand_b as i32 + 1) as u32
    }

    pub fn set_dirt<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: &BlockPos,
        below_trunk_provider: &BlockStateProvider,
    ) {
        if let Some(state) = below_trunk_provider.get_optional(block_registry, chunk, random, *pos)
        {
            chunk.set_block_state(&pos.0, state);
        }
    }

    pub fn place<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: &BlockPos,
        trunk_block: &BlockState,
    ) -> bool {
        let block = GenerationCache::get_block_state(chunk, &pos.0);
        if TreeFeature::can_replace(block.to_state(), block.to_block_id()) {
            chunk.set_block_state(&pos.0, trunk_block);
            return true;
        }
        false
    }

    pub fn try_place<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: &BlockPos,
        trunk_block: &BlockState,
    ) -> bool {
        let block = GenerationCache::get_block_state(chunk, &pos.0);
        if TreeFeature::can_replace_or_log(block.to_state(), block.to_block_id()) {
            return self.place(chunk, pos, trunk_block);
        }
        false
    }

    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        self.r#type.generate(
            block_registry,
            self,
            height,
            start_pos,
            chunk,
            random,
            below_trunk_provider,
            trunk_state,
        )
    }
}

pub enum TrunkType {
    Straight(StraightTrunkPlacer),
    Forking(ForkingTrunkPlacer),
    Giant(GiantTrunkPlacer),
    MegaJungle(MegaJungleTrunkPlacer),
    DarkOak(DarkOakTrunkPlacer),
    Fancy(FancyTrunkPlacer),
    Bending(BendingTrunkPlacer),
    UpwardsBranching(UpwardsBranchingTrunkPlacer),
    Cherry(CherryTrunkPlacer),
}

impl TrunkType {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        match self {
            Self::Straight(_) => StraightTrunkPlacer::generate(
                block_registry,
                placer,
                height,
                start_pos,
                chunk,
                random,
                below_trunk_provider,
                trunk_state,
            ),
            Self::Forking(_) => (vec![], vec![]), // TODO
            Self::Giant(_) => GiantTrunkPlacer::generate(
                block_registry,
                placer,
                height,
                start_pos,
                chunk,
                random,
                below_trunk_provider,
                trunk_state,
            ),
            Self::MegaJungle(_) => MegaJungleTrunkPlacer::generate(
                block_registry,
                placer,
                height,
                start_pos,
                chunk,
                random,
                below_trunk_provider,
                trunk_state,
            ),
            Self::DarkOak(_) => DarkOakTrunkPlacer::generate(
                block_registry,
                placer,
                height,
                start_pos,
                chunk,
                random,
                below_trunk_provider,
                trunk_state,
            ),
            Self::Fancy(_) => FancyTrunkPlacer::generate(
                block_registry,
                placer,
                height,
                start_pos,
                chunk,
                random,
                below_trunk_provider,
                trunk_state,
            ),
            Self::Bending(bending) => bending.generate(
                block_registry,
                placer,
                height,
                start_pos,
                chunk,
                random,
                below_trunk_provider,
                trunk_state,
            ),
            Self::UpwardsBranching(_) => (vec![], vec![]), // TODO
            Self::Cherry(_) => (vec![], vec![]),           // TODO
        }
    }
}
