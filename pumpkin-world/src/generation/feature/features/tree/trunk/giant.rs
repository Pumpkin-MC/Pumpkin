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
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let pos = start_pos.down();
        placer.set_dirt(block_registry, chunk, random, &pos, below_trunk_provider);
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &pos.east(),
            below_trunk_provider,
        );
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &pos.south(),
            below_trunk_provider,
        );
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &pos.south().east(),
            below_trunk_provider,
        );

        let mut trunk_poses = Vec::new();
        for y in 0..height {
            if placer.try_place(chunk, &pos.up_height(y as i32), trunk_block) {
                trunk_poses.push(pos.up_height(y as i32));
            }
            if y >= height - 1 {
                continue;
            }

            if placer.try_place(chunk, &pos.east().up_height(y as i32), trunk_block) {
                trunk_poses.push(pos.east().up_height(y as i32));
            }
            if placer.try_place(chunk, &pos.east().south().up_height(y as i32), trunk_block) {
                trunk_poses.push(pos.east().south().up_height(y as i32));
            }
            if placer.try_place(chunk, &pos.south().up_height(y as i32), trunk_block) {
                trunk_poses.push(pos.south().up_height(y as i32));
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
