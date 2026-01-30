use fancy::FancyTrunkPlacer;
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;
use straight::StraightTrunkPlacer;

use super::{TreeFeature, TreeNode};
use crate::generation::feature::features::tree::trunk::{
    bending::BendingTrunkPlacer, cherry::CherryTrunkPlacer, dark_oak::DarkOakTrunkPlacer,
    forking::ForkingTrunkPlacer, giant::GiantTrunkPlacer, mega_jungle::MegaJungleTrunkPlacer,
    upwards_branching::UpwardsBranchingTrunkPlacer,
};
use crate::generation::proto_chunk::GenerationCache;

mod bending;
mod cherry;
mod dark_oak;
mod fancy;
mod forking;
mod giant;
mod mega_jungle;
mod straight;
mod upwards_branching;

#[derive(Deserialize)]
pub struct TrunkPlacer {
    base_height: u8,
    height_rand_a: u8,
    height_rand_b: u8,
    #[serde(flatten)]
    r#type: TrunkType,
}

impl TrunkPlacer {
    pub fn get_height(&self, random: &mut RandomGenerator) -> u32 {
        self.base_height as u32
            + random.next_bounded_i32(self.height_rand_a as i32 + 1) as u32
            + random.next_bounded_i32(self.height_rand_b as i32 + 1) as u32
    }

    pub fn set_dirt<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: &BlockPos,
        force_dirt: bool,
        dirt_state: &BlockState,
    ) {
        let block = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();
        if force_dirt
            || !(tag::Block::MINECRAFT_DIRT.1.contains(&block)
                && block != Block::GRASS_BLOCK
                && block != Block::MYCELIUM)
        {
            chunk.set_block_state(&pos.0, dirt_state);
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
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        force_dirt: bool,
        dirt_state: &BlockState,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        self.r#type.generate(
            self,
            height,
            start_pos,
            chunk,
            random,
            force_dirt,
            dirt_state,
            trunk_state,
        )
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TrunkType {
    #[serde(rename = "minecraft:straight_trunk_placer")]
    Straight(StraightTrunkPlacer),
    #[serde(rename = "minecraft:forking_trunk_placer")]
    Forking(ForkingTrunkPlacer),
    #[serde(rename = "minecraft:giant_trunk_placer")]
    Giant(GiantTrunkPlacer),
    #[serde(rename = "minecraft:mega_jungle_trunk_placer")]
    MegaJungle(MegaJungleTrunkPlacer),
    #[serde(rename = "minecraft:dark_oak_trunk_placer")]
    DarkOak(DarkOakTrunkPlacer),
    #[serde(rename = "minecraft:fancy_trunk_placer")]
    Fancy(FancyTrunkPlacer),
    #[serde(rename = "minecraft:bending_trunk_placer")]
    Bending(BendingTrunkPlacer),
    #[serde(rename = "minecraft:upwards_branching_trunk_placer")]
    UpwardsBranching(UpwardsBranchingTrunkPlacer),
    #[serde(rename = "minecraft:cherry_trunk_placer")]
    Cherry(CherryTrunkPlacer),
}

impl TrunkType {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        force_dirt: bool,
        dirt_state: &BlockState,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        match self {
            Self::Straight(_) => StraightTrunkPlacer::generate(
                placer,
                height,
                start_pos,
                chunk,
                force_dirt,
                dirt_state,
                trunk_state,
            ),
            Self::Forking(_) => (vec![], vec![]), // TODO
            Self::Giant(_) => GiantTrunkPlacer::generate(
                placer,
                height,
                start_pos,
                chunk,
                random,
                force_dirt,
                dirt_state,
                trunk_state,
            ),
            Self::MegaJungle(_) => MegaJungleTrunkPlacer::generate(
                placer,
                height,
                start_pos,
                chunk,
                random,
                force_dirt,
                dirt_state,
                trunk_state,
            ),
            Self::DarkOak(_) => DarkOakTrunkPlacer::generate(
                placer,
                height,
                start_pos,
                chunk,
                random,
                force_dirt,
                dirt_state,
                trunk_state,
            ),
            Self::Fancy(_) => FancyTrunkPlacer::generate(
                placer,
                height,
                start_pos,
                chunk,
                random,
                force_dirt,
                dirt_state,
                trunk_state,
            ),
            Self::Bending(bending) => bending.generate(
                placer,
                height,
                start_pos,
                chunk,
                random,
                force_dirt,
                dirt_state,
                trunk_state,
            ),
            Self::UpwardsBranching(_) => (vec![], vec![]), // TODO
            Self::Cherry(_) => (vec![], vec![]),           // TODO
        }
    }
}
