use pumpkin_data::BlockState;
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode};

use super::TrunkPlacer;

#[derive(Deserialize)]
pub struct FancyTrunkPlacer;

impl FancyTrunkPlacer {
    pub fn generate(
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk,
        trunk_block: &BlockState,
    ) -> Vec<TreeNode> {
        // TODO
        for i in 0..height {
            placer.place(
                chunk,
                &BlockPos(start_pos.0.add_raw(0, i as i32, 0)),
                trunk_block,
            );
        }
        vec![TreeNode {
            center: start_pos.up_height(height as i32),
            foliage_radius: 0,
            giant_trunk: false,
        }]
    }
}
