use core::f32;
use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::features::tree::{
        TreeNode,
        trunk::{TrunkPlacer, giant::GiantTrunkPlacer},
    },
    level::Level,
};

#[derive(Deserialize)]
pub struct MegaJungleTrunkPlacer;

impl MegaJungleTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        force_dirt: bool,
        dirt_state: &BlockState,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let (mut nodes, mut trunk_poses) = GiantTrunkPlacer::generate(
            placer,
            height,
            start_pos,
            chunk,
            level,
            random,
            force_dirt,
            dirt_state,
            trunk_block,
        );
        let mut i = height as i32 - 2 - random.next_bounded_i32(4);

        let mut j_val = 0;
        let mut k_val = 0;

        while i > height as i32 / 2 {
            let f = random.next_f32() * (f32::consts::PI * 2.0);

            for l in 0..5 {
                j_val = (1.5f32 + f.cos() * l as f32) as i32;
                k_val = (1.5f32 + f.sin() * l as f32) as i32;

                let block_pos = start_pos.add(j_val, i - 3 + l / 2, k_val);

                if placer.try_place(chunk, &block_pos, trunk_block) {
                    trunk_poses.push(block_pos);
                }
            }

            nodes.push(TreeNode {
                center: start_pos.add(j_val, i, k_val),
                foliage_radius: -2,
                giant_trunk: false,
            });

            i -= 2 + random.next_bounded_i32(4);
        }
        (nodes, trunk_poses)
    }
}
