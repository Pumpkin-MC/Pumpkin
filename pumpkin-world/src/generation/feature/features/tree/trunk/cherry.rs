use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::int_provider::{IntProvider, UniformIntProvider};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use super::TrunkPlacer;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;
use pumpkin_data::block_properties::Axis;

pub struct CherryTrunkPlacer {
    pub branch_count: IntProvider,
    pub branch_horizontal_length: IntProvider,
    pub branch_start_offset_from_top: UniformIntProvider,
    pub branch_end_offset_from_top: IntProvider,
}

impl CherryTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
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

        let first_branch_offset =
            (height as i32 - 1 + self.branch_start_offset_from_top.get(random)).max(0);
        let second_branch_start_offset_from_top = UniformIntProvider::new(
            self.branch_start_offset_from_top.min_inclusive,
            self.branch_start_offset_from_top.max_inclusive - 1,
        );
        let mut second_branch_offset =
            (height as i32 - 1 + second_branch_start_offset_from_top.get(random)).max(0);

        if second_branch_offset >= first_branch_offset {
            second_branch_offset += 1;
        }

        let branch_count = self.branch_count.get(random);
        let has_middle_branch = branch_count == 3;
        let has_both_side_branches = branch_count >= 2;

        let trunk_height = if has_middle_branch {
            height
        } else if has_both_side_branches {
            (first_branch_offset.max(second_branch_offset) + 1) as u32
        } else {
            (first_branch_offset + 1) as u32
        };

        for y in 0..trunk_height {
            let pos = start_pos.up_height(y as i32);
            if placer.place(chunk, &pos, trunk_state) {
                logs.push(pos);
            }
        }

        let mut nodes = Vec::new();
        if has_middle_branch {
            nodes.push(TreeNode {
                center: start_pos.up_height(trunk_height as i32),
                foliage_radius: 0,
                giant_trunk: false,
            });
        }

        let directions = [
            BlockPos::new(0, 0, -1), // North
            BlockPos::new(0, 0, 1),  // South
            BlockPos::new(-1, 0, 0), // West
            BlockPos::new(1, 0, 0),  // East
        ];
        let branch_direction = directions[random.next_bounded_i32(4) as usize];

        let (node1, logs1) = self.generate_branch(
            placer,
            height,
            start_pos,
            chunk,
            random,
            trunk_state,
            branch_direction,
            first_branch_offset,
            first_branch_offset < trunk_height as i32 - 1,
        );
        nodes.push(node1);
        logs.extend(logs1);

        if has_both_side_branches {
            let opposite_direction = BlockPos::new(-branch_direction.0.x, 0, -branch_direction.0.z);
            let (node2, logs2) = self.generate_branch(
                placer,
                height,
                start_pos,
                chunk,
                random,
                trunk_state,
                opposite_direction,
                second_branch_offset,
                second_branch_offset < trunk_height as i32 - 1,
            );
            nodes.push(node2);
            logs.extend(logs2);
        }

        (nodes, logs)
    }

    #[expect(clippy::too_many_arguments)]
    fn generate_branch<T: GenerationCache>(
        &self,
        placer: &TrunkPlacer,
        tree_height: u32,
        origin: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        trunk_state: &BlockState,
        branch_direction: BlockPos,
        offset_from_origin: i32,
        middle_continues_upwards: bool,
    ) -> (TreeNode, Vec<BlockPos>) {
        let mut logs = Vec::new();
        let mut log_pos = origin.up_height(offset_from_origin);

        let branch_end_pos_offset =
            tree_height as i32 - 1 + self.branch_end_offset_from_top.get(random);
        let extend_branch_away_from_trunk =
            middle_continues_upwards || branch_end_pos_offset < offset_from_origin;
        let distance_to_trunk =
            self.branch_horizontal_length.get(random) + i32::from(extend_branch_away_from_trunk);

        let branch_end_pos = origin
            .add(
                branch_direction.0.x * distance_to_trunk,
                0,
                branch_direction.0.z * distance_to_trunk,
            )
            .up_height(branch_end_pos_offset);

        let steps_horizontally = if extend_branch_away_from_trunk { 2 } else { 1 };

        let axis = if branch_direction.0.x != 0 {
            Axis::X
        } else {
            Axis::Z
        };
        let sideways_state = self.get_sideways_state(trunk_state, axis);

        for _ in 0..steps_horizontally {
            log_pos = log_pos.add(branch_direction.0.x, 0, branch_direction.0.z);
            if placer.place(chunk, &log_pos, sideways_state) {
                logs.push(log_pos);
            }
        }

        let vertical_direction = if branch_end_pos.0.y > log_pos.0.y {
            1
        } else {
            -1
        };

        loop {
            let distance = log_pos.manhattan_distance(branch_end_pos);

            if distance == 0 {
                return (
                    TreeNode {
                        center: branch_end_pos.up(),
                        foliage_radius: 0,
                        giant_trunk: false,
                    },
                    logs,
                );
            }

            let chance_to_grow_vertically =
                (branch_end_pos.0.y - log_pos.0.y).abs() as f32 / distance as f32;
            let grow_vertically = random.next_f32() < chance_to_grow_vertically;

            if grow_vertically {
                log_pos = log_pos.up_height(vertical_direction);
                if placer.place(chunk, &log_pos, trunk_state) {
                    logs.push(log_pos);
                }
            } else {
                log_pos = log_pos.add(branch_direction.0.x, 0, branch_direction.0.z);
                if placer.place(chunk, &log_pos, sideways_state) {
                    logs.push(log_pos);
                }
            }
        }
    }

    fn get_sideways_state(&self, trunk_state: &BlockState, axis: Axis) -> &'static BlockState {
        let block = Block::from_state_id(trunk_state.id);
        if let Some(props_source) = block.properties(trunk_state.id) {
            let mut props = props_source.to_props();
            let axis_str = match axis {
                Axis::X => "x",
                Axis::Y => "y",
                Axis::Z => "z",
            };

            if let Some(idx) = props.iter().position(|(k, _)| *k == "axis") {
                props[idx] = ("axis", axis_str);
            } else {
                props.push(("axis", axis_str));
            }

            let new_state_id = block.from_properties(&props).to_state_id(block);
            return BlockState::from_id(new_state_id);
        }
        BlockState::from_id(trunk_state.id)
    }
}
