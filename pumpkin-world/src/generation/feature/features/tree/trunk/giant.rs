use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::{
        block_state_provider::BlockStateProvider,
        feature::features::tree::{TreeNode, trunk::TrunkPlacer},
    },
    world::BlockRegistryExt,
};

pub struct GiantTrunkPlacer;

impl GiantTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        block_registry: &dyn BlockRegistryExt,
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
            &pos.offset(BlockDirection::East.to_offset()),
            below_trunk_provider,
        );
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &pos.offset(BlockDirection::South.to_offset()),
            below_trunk_provider,
        );
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &pos.offset(BlockDirection::South.to_offset())
                .offset(BlockDirection::South.to_offset()),
            below_trunk_provider,
        );

        let mut trunk_poses = Vec::new();
        for y in 0..height {
            if placer.try_place(
                chunk,
                &pos.offset(Vector3::new(0, y as i32, 0)),
                trunk_block,
            ) {
                trunk_poses.push(pos.offset(Vector3::new(0, y as i32, 0)));
            }
            if y >= height - 1 {
                continue;
            }
            if placer.try_place(
                chunk,
                &pos.offset(Vector3::new(1, y as i32, 0)),
                trunk_block,
            ) {
                trunk_poses.push(pos.offset(Vector3::new(1, y as i32, 0)));
            }
            if placer.try_place(
                chunk,
                &pos.offset(Vector3::new(1, y as i32, 1)),
                trunk_block,
            ) {
                trunk_poses.push(pos.offset(Vector3::new(1, y as i32, 1)));
            }
            if placer.try_place(
                chunk,
                &pos.offset(Vector3::new(0, y as i32, 1)),
                trunk_block,
            ) {
                trunk_poses.push(pos.offset(Vector3::new(0, y as i32, 1)));
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
