use pumpkin_data::BlockState;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::{
        block_state_provider::BlockStateProvider,
        feature::features::tree::{TreeNode, trunk::TrunkPlacer},
    },
    world::WorldPortalExt,
};

pub struct GiantTrunkPlacer;

impl GiantTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        block_registry: &dyn WorldPortalExt,
        _placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let below = start_pos.down();
        TrunkPlacer::set_dirt(block_registry, chunk, random, &below, below_trunk_provider);
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &below.east(),
            below_trunk_provider,
        );
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &below.south(),
            below_trunk_provider,
        );
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &below.south().east(),
            below_trunk_provider,
        );

        let mut trunk_poses = Vec::new();
        for y in 0..height {
            let log_pos = start_pos.up_height(y as i32);
            if TrunkPlacer::try_place(chunk, &log_pos, trunk_block) {
                trunk_poses.push(log_pos);
            }
            if y >= height - 1 {
                continue;
            }

            let log_pos_east = start_pos.east().up_height(y as i32);
            if TrunkPlacer::try_place(chunk, &log_pos_east, trunk_block) {
                trunk_poses.push(log_pos_east);
            }
            let log_pos_se = start_pos.east().south().up_height(y as i32);
            if TrunkPlacer::try_place(chunk, &log_pos_se, trunk_block) {
                trunk_poses.push(log_pos_se);
            }
            let log_pos_south = start_pos.south().up_height(y as i32);
            if TrunkPlacer::try_place(chunk, &log_pos_south, trunk_block) {
                trunk_poses.push(log_pos_south);
            }
        }
        (
            vec![TreeNode {
                center: start_pos.up_height(height as i32),
                foliage_radius: 0,
                giant_trunk: true,
            }],
            trunk_poses,
        )
    }
}
