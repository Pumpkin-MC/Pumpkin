use pumpkin_data::BlockState;
use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode};

use super::TrunkPlacer;

#[derive(Deserialize)]
pub struct StraightTrunkPlacer;

impl StraightTrunkPlacer {
    pub fn generate(
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let mut logs = Vec::new();
        for i in 0..height {
            let pos = start_pos.up_height(i as i32);
            if placer.place(chunk, &pos, trunk_block) {
                logs.push(pos);
            }
        }
        (
            vec![TreeNode {
                center: start_pos.up_height(height as i32),
                foliage_radius: 0,
                giant_trunk: false,
            }],
            logs,
        )
    }
}
