use std::sync::Arc;

use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::features::tree::{TreeFeature, TreeNode, trunk::TrunkPlacer},
    level::Level,
};

#[derive(Deserialize)]
pub struct DarkOakTrunkPlacer;

impl DarkOakTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
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
        let pos = start_pos.down();
        placer.set_dirt(chunk, &pos, force_dirt, dirt_state);
        placer.set_dirt(
            chunk,
            &pos.offset(BlockDirection::East.to_offset()),
            force_dirt,
            dirt_state,
        );
        placer.set_dirt(
            chunk,
            &pos.offset(BlockDirection::South.to_offset()),
            force_dirt,
            dirt_state,
        );
        placer.set_dirt(
            chunk,
            &pos.offset(BlockDirection::South.to_offset())
                .offset(BlockDirection::South.to_offset()),
            force_dirt,
            dirt_state,
        );
        let y_height = start_pos.0.y + height as i32 - 1;
        let max_height = height - random.next_bounded_i32(4) as u32;
        let mut trunk_poses = Vec::new();
        let mut nodes = Vec::new();
        let mut rand = random.next_bounded_i32(3);

        let mut x = pos.0.x;
        let mut z = pos.0.x;

        // TODO: make this random
        let random_direction = BlockDirection::North;

        for y in 0..height {
            if y >= max_height && rand > 0 {
                x += random_direction.to_offset().x;
                z += random_direction.to_offset().z;
                rand -= 1;
            }
            let pos = BlockPos::new(x, y_height, z);
            // TODO: support multiple chunks
            let state = chunk.get_block_state(&pos.0);
            if !TreeFeature::is_air_or_leaves(&state.to_state(), &state.to_block()) {
                continue;
            }
            if placer.try_place(chunk, &pos, trunk_block) {
                trunk_poses.push(pos);
            }
            if placer.try_place(
                chunk,
                &pos.offset(BlockDirection::East.to_offset()),
                trunk_block,
            ) {
                trunk_poses.push(pos.offset(BlockDirection::East.to_offset()));
            }
            if placer.try_place(
                chunk,
                &pos.offset(BlockDirection::South.to_offset()),
                trunk_block,
            ) {
                trunk_poses.push(pos.offset(BlockDirection::South.to_offset()));
            }
            if placer.try_place(
                chunk,
                &pos.offset(BlockDirection::East.to_offset())
                    .offset(BlockDirection::South.to_offset()),
                trunk_block,
            ) {
                trunk_poses.push(
                    pos.offset(BlockDirection::East.to_offset())
                        .offset(BlockDirection::South.to_offset()),
                );
            }
        }
        nodes.push(TreeNode {
            center: BlockPos::new(x, y_height, z),
            foliage_radius: 0,
            giant_trunk: true,
        });
        for xd in -1..2 {
            for zd in -1..2 {
                if (0..=1).contains(&xd) && (0..=1).contains(&zd) || random.next_bounded_i32(3) > 0
                {
                    continue;
                }
                let h = random.next_bounded_i32(3) + 2;
                for height in 0..h {
                    if placer.try_place(
                        chunk,
                        &BlockPos::new(
                            start_pos.0.x + xd,
                            y_height - height - 1,
                            start_pos.0.z + zd,
                        ),
                        trunk_block,
                    ) {
                        trunk_poses.push(BlockPos::new(
                            start_pos.0.x + xd,
                            y_height - height - 1,
                            start_pos.0.z + zd,
                        ));
                    }
                }
                nodes.push(TreeNode {
                    center: BlockPos::new(start_pos.0.x + xd, y_height, start_pos.0.z + zd),
                    foliage_radius: 0,
                    giant_trunk: true,
                });
            }
        }

        (nodes, trunk_poses)
    }
}
