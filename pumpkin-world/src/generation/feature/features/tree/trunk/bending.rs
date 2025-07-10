use std::sync::Arc;

use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::features::tree::{TreeNode, trunk::TrunkPlacer},
    level::Level,
};

#[derive(Deserialize)]
pub struct BendingTrunkPlacer {
    min_height_for_leaves: u32,
    bend_length: IntProvider,
}

impl BendingTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk<'_>,
        _level: &Arc<Level>,
        random: &mut RandomGenerator,
        force_dirt: bool,
        dirt_state: &BlockState,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        placer.set_dirt(chunk, &start_pos.down(), force_dirt, dirt_state);

        // TODO: make this random
        let random_direction = BlockDirection::North;
        let height = height - 1;
        let mut pos = start_pos;
        let mut trunk_poses = Vec::new();
        let mut nodes = Vec::new();
        for y in 0..height {
            if y + 1 >= height + random.next_bounded_i32(2) as u32 {
                pos = pos.offset(random_direction.to_offset());
            }
            if placer.place(chunk, &pos, trunk_block) {
                trunk_poses.push(pos);
            }
            if y >= 1 {
                nodes.push(TreeNode {
                    center: pos,
                    foliage_radius: 0,
                    giant_trunk: false,
                });
            }
            pos = pos.up();
        }
        let bend = self.bend_length.get(random);
        for _ in 0..bend {
            if placer.place(chunk, &pos, trunk_block) {
                trunk_poses.push(pos);
            }
            nodes.push(TreeNode {
                center: pos,
                foliage_radius: 0,
                giant_trunk: false,
            });
            pos = pos.offset(random_direction.to_offset());
        }
        (nodes, trunk_poses)
    }
}
