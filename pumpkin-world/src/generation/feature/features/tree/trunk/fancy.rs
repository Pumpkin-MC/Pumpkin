use core::f32;
use std::sync::Arc;

use pumpkin_data::{
    BlockState,
    block_properties::{Axis, EnumVariants, get_block_by_state_id, get_state_by_state_id},
};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::feature::features::tree::{TreeFeature, TreeNode},
    level::Level,
};

use super::TrunkPlacer;

#[derive(Deserialize)]
pub struct FancyTrunkPlacer;

impl FancyTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        force_dirt: bool,
        dirt_state: &BlockState,
        trunk_block: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        let j = height as i32 + 2;
        let k = ((j as f64) * 0.618).floor() as i32;

        placer.set_dirt(chunk, &start_pos.down(), force_dirt, dirt_state);

        let l = ((1.382 + (1.0 * (j as f64) / 13.0).powf(2.0)).floor() as i32).min(1);
        let m = start_pos.0.y + k;
        let mut list: Vec<BranchPosition> = Vec::new();
        let mut logs = Vec::new();

        list.push(BranchPosition::new(start_pos, m));

        for n in (0..=(j - 5)).rev() {
            let f = Self::should_generate_branch(j, n);
            if f < 0.0f32 {
                continue;
            }

            for _o in 0..l {
                let g = (1.0f32 * f * (random.next_f32() + 0.328f32)) as f64;
                let h = (random.next_f32() * 2.0f32 * f32::consts::PI) as f64;
                let p = g * h.sin() + 0.5f64;
                let q = g * h.cos() + 0.5f64;

                let block_pos = start_pos.add(p.floor() as i32, n - 1, q.floor() as i32);
                let block_pos_2 = block_pos.up_height(5);

                let (i, new_logs) = Self::make_or_check_branch(
                    chunk,
                    level,
                    block_pos.0,
                    block_pos_2.0,
                    trunk_block,
                    false,
                );
                logs.extend_from_slice(&new_logs);

                if !i {
                    continue;
                }

                let r = start_pos.0.x - block_pos.0.x;
                let s = start_pos.0.z - block_pos.0.z;
                let t = (block_pos.0.y as f64) - ((r * r + s * s) as f64).sqrt() * 0.381f64;
                let u = if t > (m as f64) { m } else { t as i32 };

                let block_pos_3 = BlockPos::new(start_pos.0.x, u, start_pos.0.z);

                let (i, new_logs) = Self::make_or_check_branch(
                    chunk,
                    level,
                    block_pos_3.0,
                    block_pos.0,
                    trunk_block,
                    false,
                );
                logs.extend_from_slice(&new_logs);

                if !i {
                    continue;
                }
                list.push(BranchPosition::new(block_pos, block_pos_3.0.y));
            }
        }

        Self::make_or_check_branch(
            chunk,
            level,
            start_pos.0,
            start_pos.up_height(k).0,
            trunk_block,
            true,
        );
        Self::make_branches(chunk, level, j, start_pos.0, trunk_block, &list);

        let mut list_2: Vec<TreeNode> = Vec::new();
        for branch_position in list {
            if Self::is_high_enough(j, branch_position.get_end_y() - start_pos.0.y) {
                list_2.push(branch_position.node);
            }
        }
        (list_2, logs)
    }

    fn make_or_check_branch(
        chunk: &mut ProtoChunk<'_>,
        _level: &Arc<Level>,
        start_pos: Vector3<i32>,
        branch_pos: Vector3<i32>,
        trunk_provider: &BlockState,
        make: bool,
    ) -> (bool, Vec<BlockPos>) {
        if !make && start_pos == branch_pos {
            return (true, vec![]);
        }

        let block_pos_offset = Vector3::new(
            branch_pos.x - start_pos.x,
            branch_pos.y - start_pos.y,
            branch_pos.z - start_pos.z,
        );
        let i = Self::get_longest_side(block_pos_offset);

        let f = block_pos_offset.x as f32 / i as f32;
        let g = block_pos_offset.y as f32 / i as f32;
        let h = block_pos_offset.z as f32 / i as f32;

        let mut logs = Vec::new();
        for j in 0..=i {
            let block_pos_2 = BlockPos(start_pos.add_raw(
                (0.5f32 + j as f32 * f).floor() as i32,
                (0.5f32 + j as f32 * g).floor() as i32,
                (0.5f32 + j as f32 * h).floor() as i32,
            ));

            let block = chunk.get_block_state(&block_pos_2.0);

            if make {
                let axis = Self::get_log_axis(start_pos, block_pos_2.0);

                if TreeFeature::can_replace(block.to_state(), block.to_block()) {
                    let block = get_block_by_state_id(trunk_provider.id);
                    let original_props = &block.properties(trunk_provider.id).unwrap().to_props();
                    let axis = axis.to_value();
                    // Set the right Axis
                    let props = original_props
                        .iter()
                        .map(|(key, value)| {
                            if key == "axis" {
                                (key.as_str(), axis)
                            } else {
                                (key.as_str(), value.as_str())
                            }
                        })
                        .collect();
                    let state = block.from_properties(props).unwrap().to_state_id(block);
                    if chunk.chunk_pos == block_pos_2.chunk_and_chunk_relative_position().0 {
                        chunk.set_block_state(&block_pos_2.0, get_state_by_state_id(state));
                    } else {
                        // level.set_block_state(&block_pos_2, state).await;
                    }
                    logs.push(block_pos_2);
                    continue;
                }
            }

            if TreeFeature::can_replace_or_log(block.to_state(), block.to_block()) {
                continue;
            }
            return (false, logs);
        }
        (true, logs)
    }

    fn make_branches(
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        tree_height: i32,
        start_pos: Vector3<i32>,
        trunk_provider: &BlockState,
        branch_positions: &Vec<BranchPosition>,
    ) {
        for branch_position in branch_positions {
            let i = branch_position.get_end_y();
            let block_pos = BlockPos::new(start_pos.x, i, start_pos.z);
            if block_pos == branch_position.node.center
                || !Self::is_high_enough(tree_height, i - start_pos.y)
            {
                continue;
            }
            Self::make_or_check_branch(
                chunk,
                level,
                block_pos.0,
                branch_position.node.center.0,
                trunk_provider,
                true,
            );
        }
    }

    fn should_generate_branch(tree_height: i32, height: i32) -> f32 {
        if (height as f32) < (tree_height as f32) * 0.3f32 {
            return -1.0f32;
        }
        let f = (tree_height as f32) / 2.0f32;
        let g = f - (height as f32);
        let h = (f * f - g * g).sqrt();
        if g == 0.0f32 {
            h
        } else if g.abs() >= f {
            0.0f32
        } else {
            h * 0.5f32
        }
    }

    fn get_longest_side(offset: Vector3<i32>) -> i32 {
        let x = offset.x.abs();
        let y = offset.y.abs();
        let z = offset.z.abs();
        x.max(y.max(z))
    }

    fn get_log_axis(branch_start: Vector3<i32>, branch_end: Vector3<i32>) -> Axis {
        let x = (branch_end.x - branch_start.x).abs();
        let z = (branch_end.z - branch_start.z).abs();
        let max = x.max(z);
        if max > 0 {
            if x == max { Axis::X } else { Axis::Z }
        } else {
            Axis::Y
        }
    }

    fn is_high_enough(tree_height: i32, height: i32) -> bool {
        (height as f64) >= (tree_height as f64) * 0.2
    }
}

pub struct BranchPosition {
    pub node: TreeNode,
    end_y: i32,
}

impl BranchPosition {
    pub fn new(pos: BlockPos, end_y: i32) -> Self {
        BranchPosition {
            node: TreeNode {
                center: pos,
                foliage_radius: 0,
                giant_trunk: false,
            },
            end_y,
        }
    }

    pub fn get_end_y(&self) -> i32 {
        self.end_y
    }
}
