use pumpkin_data::BlockState;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;

use super::TrunkPlacer;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

pub struct StraightTrunkPlacer;

impl StraightTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        block_registry: &dyn BlockRegistryExt,
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &start_pos.down(),
            below_trunk_provider,
        );
        let mut logs = Vec::new();
        for i in 0..height {
            let pos = start_pos.up_height(i as i32);
            if placer.place(chunk, &pos, trunk_state) {
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
