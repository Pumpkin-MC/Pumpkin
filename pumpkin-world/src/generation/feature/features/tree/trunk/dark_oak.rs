use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{
    generation::{
        block_state_provider::BlockStateProvider,
        feature::features::tree::{TreeFeature, TreeNode, trunk::TrunkPlacer},
    },
    world::WorldPortalExt,
};

pub struct DarkOakTrunkPlacer;

impl DarkOakTrunkPlacer {
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
        let below = start_pos.down();
        placer.set_dirt(block_registry, chunk, random, &below, below_trunk_provider);
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &below.east(),
            below_trunk_provider,
        );
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &below.south(),
            below_trunk_provider,
        );
        placer.set_dirt(
            block_registry,
            chunk,
            random,
            &below.south().east(),
            below_trunk_provider,
        );

        let horizontal_directions = BlockDirection::horizontal();
        let lean_direction = horizontal_directions[random.next_bounded_i32(4) as usize];
        let lean_height = height as i32 - random.next_bounded_i32(4);
        let mut lean_steps = 2 - random.next_bounded_i32(3);

        let mut tx = start_pos.0.x;
        let mut tz = start_pos.0.z;
        let ey = start_pos.0.y + height as i32 - 1;

        let mut trunk_poses = Vec::new();
        for dy in 0..height as i32 {
            if dy >= lean_height && lean_steps > 0 {
                let offset = lean_direction.to_offset();
                tx += offset.x;
                tz += offset.z;
                lean_steps -= 1;
            }

            let yy = start_pos.0.y + dy;
            let pos = BlockPos::new(tx, yy, tz);

            // Check if air or leaves at the main 2x2 position
            let state = GenerationCache::get_block_state(chunk, &pos.0);
            if TreeFeature::is_air_or_leaves(state.to_state(), state.to_block_id()) {
                if placer.try_place(chunk, &pos, trunk_block) {
                    trunk_poses.push(pos);
                }
                if placer.try_place(chunk, &pos.east(), trunk_block) {
                    trunk_poses.push(pos.east());
                }
                if placer.try_place(chunk, &pos.south(), trunk_block) {
                    trunk_poses.push(pos.south());
                }
                if placer.try_place(chunk, &pos.east().south(), trunk_block) {
                    trunk_poses.push(pos.east().south());
                }
            }
        }

        let mut nodes = Vec::new();
        nodes.push(TreeNode {
            center: BlockPos::new(tx, ey, tz),
            foliage_radius: 0,
            giant_trunk: true,
        });

        for ox in -1..=2 {
            for oz in -1..=2 {
                if (!(0..=1).contains(&ox) || !(0..=1).contains(&oz))
                    && random.next_bounded_i32(3) <= 0
                {
                    let length = random.next_bounded_i32(3) + 2;

                    for branch_y in 0..length {
                        let pos = BlockPos::new(
                            start_pos.0.x + ox,
                            ey - branch_y - 1,
                            start_pos.0.z + oz,
                        );
                        if placer.try_place(chunk, &pos, trunk_block) {
                            trunk_poses.push(pos);
                        }
                    }

                    nodes.push(TreeNode {
                        center: BlockPos::new(start_pos.0.x + ox, ey, start_pos.0.z + oz),
                        foliage_radius: 0,
                        giant_trunk: false,
                    });
                }
            }
        }

        (nodes, trunk_poses)
    }
}
