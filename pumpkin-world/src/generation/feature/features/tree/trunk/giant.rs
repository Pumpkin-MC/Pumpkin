use std::sync::Arc;

use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::features::tree::{TreeNode, trunk::TrunkPlacer},
    level::Level,
};

#[derive(Deserialize)]
pub struct GiantTrunkPlacer;

impl GiantTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk<'_>,
        _level: &Arc<Level>,
        _random: &mut RandomGenerator,
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
