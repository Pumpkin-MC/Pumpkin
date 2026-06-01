use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::{
        block_state_provider::BlockStateProvider,
        feature::features::tree::{TreeNode, trunk::TrunkPlacer},
    },
    world::WorldPortalExt,
};

pub struct BendingTrunkPlacer {
    pub min_height_for_leaves: u32,
    pub bend_length: IntProvider,
}

impl BendingTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let horizontal_directions = BlockDirection::horizontal();
        let direction = horizontal_directions[random.next_bounded_i32(4) as usize];
        let log_height = height as i32 - 1;
        let mut pos = start_pos;

        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &pos.down(),
            below_trunk_provider,
        );

        let mut trunk_poses = Vec::new();
        let mut nodes = Vec::new();

        for i in 0..=log_height {
            if i + 1 >= log_height + random.next_bounded_i32(2) {
                pos = pos.offset(direction.to_offset());
            }

            if placer.place(chunk, &pos, trunk_block) {
                trunk_poses.push(pos);
            }

            if i >= self.min_height_for_leaves as i32 {
                nodes.push(TreeNode {
                    center: pos,
                    foliage_radius: 0,
                    giant_trunk: false,
                });
            }

            pos = pos.up();
        }

        let dir_length = self.bend_length.get(random);

        for _ in 0..=dir_length {
            if placer.place(chunk, &pos, trunk_block) {
                trunk_poses.push(pos);
            }

            nodes.push(TreeNode {
                center: pos,
                foliage_radius: 0,
                giant_trunk: false,
            });
            pos = pos.offset(direction.to_offset());
        }

        (nodes, trunk_poses)
    }
}
