use pumpkin_data::block::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::ProtoChunk;

use super::placer::TrunkPlacer;

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
    pub fn generate(
        &self,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk,
        trunk_block: &BlockState,
    ) {
        match self {
            Self::Straight => {
                StraightTrunkPlacer::generate(placer, height, start_pos, chunk, trunk_block)
            }
            TrunkType::Forking => todo!(),
            TrunkType::Giant => todo!(),
            TrunkType::MegaJungle => todo!(),
            TrunkType::DarkOak => todo!(),
            TrunkType::Fancy => todo!(),
            TrunkType::Bending => todo!(),
            TrunkType::UpwardsBranching => todo!(),
            TrunkType::Cherry => todo!(),
        }
    }
}

#[derive(Deserialize)]
pub struct StraightTrunkPlacer;

impl StraightTrunkPlacer {
    pub fn generate(
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk,
        trunk_block: &BlockState,
    ) {
        for i in 0..height {
            placer.place(
                chunk,
                &BlockPos(start_pos.0.add_raw(0, i as i32, 0)),
                trunk_block,
            );
        }
    }
}
