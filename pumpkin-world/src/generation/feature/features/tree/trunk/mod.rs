use std::sync::Arc;

use fancy::FancyTrunkPlacer;
use pumpkin_data::BlockState;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;
use straight::StraightTrunkPlacer;

use crate::{ProtoChunk, level::Level};

use super::{TreeFeature, TreeNode};

mod fancy;
mod straight;

#[derive(Deserialize)]
pub struct TrunkPlacer {
    base_height: u8,
    height_rand_a: u8,
    height_rand_b: u8,
    r#type: TrunkType,
}

impl TrunkPlacer {
    pub fn get_height(&self, random: &mut RandomGenerator) -> u32 {
        self.base_height as u32
            + random.next_bounded_i32(self.height_rand_a as i32 + 1) as u32
            + random.next_bounded_i32(self.height_rand_b as i32 + 1) as u32
    }

    pub fn place(
        &self,
        chunk: &mut ProtoChunk<'_>,
        pos: &BlockPos,
        trunk_block: &BlockState,
    ) -> bool {
        let block = chunk.get_block_state(&pos.0);
        if TreeFeature::can_replace(&block.to_state(), &block.to_block()) {
            chunk.set_block_state(&pos.0, trunk_block);
            return true;
        }
        false
    }

    pub async fn generate(
        &self,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        self.r#type
            .generate(self, height, start_pos, chunk, level, random, trunk_block)
            .await
    }
}

#[derive(Deserialize)]
pub enum TrunkType {
    #[serde(rename = "minecraft:straight_trunk_placer")]
    Straight,
    #[serde(rename = "minecraft:forking_trunk_placer")]
    Forking,
    #[serde(rename = "minecraft:giant_trunk_placer")]
    Giant,
    #[serde(rename = "minecraft:mega_jungle_trunk_placer")]
    MegaJungle,
    #[serde(rename = "minecraft:dark_oak_trunk_placer")]
    DarkOak,
    #[serde(rename = "minecraft:fancy_trunk_placer")]
    Fancy,
    #[serde(rename = "minecraft:bending_trunk_placer")]
    Bending,
    #[serde(rename = "minecraft:upwards_branching_trunk_placer")]
    UpwardsBranching,
    #[serde(rename = "minecraft:cherry_trunk_placer")]
    Cherry,
}

impl TrunkType {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        match self {
            Self::Straight => {
                StraightTrunkPlacer::generate(placer, height, start_pos, chunk, trunk_block)
            }
            TrunkType::Forking => (vec![], vec![]),    // TODO
            TrunkType::Giant => (vec![], vec![]),      // TODO
            TrunkType::MegaJungle => (vec![], vec![]), // TODO
            TrunkType::DarkOak => (vec![], vec![]),    // TODO
            TrunkType::Fancy => {
                FancyTrunkPlacer::generate(
                    placer,
                    height,
                    start_pos,
                    chunk,
                    level,
                    random,
                    trunk_block,
                )
                .await
            }
            TrunkType::Bending => (vec![], vec![]), // TODO
            TrunkType::UpwardsBranching => (vec![], vec![]), // TODO
            TrunkType::Cherry => (vec![], vec![]),  // TODO
        }
    }
}
