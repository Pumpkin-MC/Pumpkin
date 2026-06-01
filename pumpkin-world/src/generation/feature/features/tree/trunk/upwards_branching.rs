use pumpkin_data::BlockState;
use pumpkin_util::math::int_provider::IntProvider;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use super::TrunkPlacer;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::{TreeFeature, TreeNode};
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;
use pumpkin_data::BlockDirection;

pub struct UpwardsBranchingTrunkPlacer {
    pub extra_branch_steps: IntProvider,
    pub place_branch_per_log_probability: f32,
    pub extra_branch_length: IntProvider,
    pub can_grow_through: &'static [u16],
}

impl UpwardsBranchingTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        _block_registry: &dyn WorldPortalExt,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        _below_trunk_provider: &BlockStateProvider,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let mut nodes = Vec::new();
        let mut logs = Vec::new();

        for height_pos in 0..height as i32 {
            let current_pos = start_pos.up_height(height_pos);
            if self.place_log_special(chunk, &current_pos, trunk_state) {
                logs.push(current_pos);
                if height_pos < height as i32 - 1
                    && random.next_f32() < self.place_branch_per_log_probability
                {
                    let branch_dir =
                        BlockDirection::horizontal()[random.next_bounded_i32(4) as usize];
                    let branch_len = self.extra_branch_length.get(random);
                    let branch_pos = (branch_len - self.extra_branch_length.get(random) - 1).max(0);
                    let branch_steps = self.extra_branch_steps.get(random);

                    self.place_branch(
                        placer,
                        height,
                        chunk,
                        random,
                        trunk_state,
                        &mut nodes,
                        &mut logs,
                        current_pos,
                        branch_dir,
                        branch_pos,
                        branch_steps,
                    );
                }
            }

            if height_pos == height as i32 - 1 {
                nodes.push(TreeNode {
                    center: current_pos.up(),
                    foliage_radius: 0,
                    giant_trunk: false,
                });
            }
        }

        (nodes, logs)
    }

    #[expect(clippy::too_many_arguments)]
    fn place_branch<T: GenerationCache>(
        &self,
        _placer: &TrunkPlacer,
        tree_height: u32,
        chunk: &mut T,
        _random: &mut RandomGenerator,
        trunk_state: &BlockState,
        nodes: &mut Vec<TreeNode>,
        logs: &mut Vec<BlockPos>,
        current_pos: BlockPos,
        branch_dir: BlockDirection,
        branch_pos: i32,
        mut branch_steps: i32,
    ) {
        let mut height_along_branch = current_pos.0.y + branch_pos;
        let mut log_x = current_pos.0.x;
        let mut log_z = current_pos.0.z;
        let mut branch_placement_index = branch_pos;

        while branch_placement_index < tree_height as i32 && branch_steps > 0 {
            if branch_placement_index >= 1 {
                let placement_height = current_pos.0.y + branch_placement_index;
                let offset = branch_dir.to_offset();
                log_x += offset.x;
                log_z += offset.z;
                height_along_branch = placement_height;

                let pos = BlockPos::new(log_x, placement_height, log_z);
                if self.place_log_special(chunk, &pos, trunk_state) {
                    logs.push(pos);
                    height_along_branch = placement_height + 1;
                }
                nodes.push(TreeNode {
                    center: pos,
                    foliage_radius: 0,
                    giant_trunk: false,
                });
            }
            branch_placement_index += 1;
            branch_steps -= 1;
        }

        if height_along_branch - current_pos.0.y > 1 {
            let foliage_pos = BlockPos::new(log_x, height_along_branch, log_z);
            nodes.push(TreeNode {
                center: foliage_pos,
                foliage_radius: 0,
                giant_trunk: false,
            });
            nodes.push(TreeNode {
                center: foliage_pos.down_height(2),
                foliage_radius: 0,
                giant_trunk: false,
            });
        }
    }

    fn place_log_special<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: &BlockPos,
        trunk_state: &BlockState,
    ) -> bool {
        let block = GenerationCache::get_block_state(chunk, &pos.0);
        if TreeFeature::can_replace(block.to_state(), block.to_block_id())
            || self.can_grow_through.contains(&block.to_block_id())
        {
            chunk.set_block_state(&pos.0, trunk_state);
            return true;
        }
        false
    }
}
