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
        // Vanilla GiantTrunkPlacer: dirt under start_pos; logs from start_pos up.
        // (Previously used start_pos.down() as log base → canopy floated above trunk.)
        let dirt = start_pos.down();
        TrunkPlacer::set_dirt(block_registry, chunk, random, &dirt, below_trunk_provider);
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &dirt.east(),
            below_trunk_provider,
        );
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &dirt.south(),
            below_trunk_provider,
        );
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &dirt.south().east(),
            below_trunk_provider,
        );

        let mut trunk_poses = Vec::new();
        for y in 0..height {
            let log = start_pos.up_height(y as i32);
            if TrunkPlacer::try_place(chunk, &log, trunk_block) {
                trunk_poses.push(log);
            }
            if TrunkPlacer::try_place(chunk, &log.east(), trunk_block) {
                trunk_poses.push(log.east());
            }
            if TrunkPlacer::try_place(chunk, &log.south(), trunk_block) {
                trunk_poses.push(log.south());
            }
            if TrunkPlacer::try_place(chunk, &log.east().south(), trunk_block) {
                trunk_poses.push(log.east().south());
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
