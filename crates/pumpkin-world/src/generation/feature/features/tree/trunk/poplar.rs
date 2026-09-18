use pumpkin_data::block_properties::{Axis, PaleOakWoodLikeProperties};
use pumpkin_data::{BlockDirection, BlockState, BlockStateId};
use pumpkin_util::math::int_provider::IntProvider;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use super::TrunkPlacer;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct PoplarTrunkPlacer {
    pub trunk_height_above_branches: IntProvider,
    pub branch_amount: IntProvider,
}

impl PoplarTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        _placer: &TrunkPlacer,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
        below_trunk_provider: &BlockStateProvider,
        trunk_state: &BlockState,
    ) -> (Vec<TreeNode>, Vec<BlockPos>) {
        TrunkPlacer::set_dirt(
            block_registry,
            chunk,
            random,
            &start_pos.down(),
            below_trunk_provider,
        );

        let mut logs = Vec::new();
        let branch_height = height as i32 - self.trunk_height_above_branches.get(random);

        for y in 0..height as i32 {
            let pos = start_pos.up_height(y);
            if TrunkPlacer::place(chunk, &pos, trunk_state) {
                logs.push(pos);
            }

            // The directions are shuffled on every layer, branching or not
            let directions = Self::shuffled_directions(random);
            if branch_height - 1 != y {
                continue;
            }

            let branch_count = self.branch_amount.get(random).max(0) as usize;
            for direction in directions.into_iter().take(branch_count) {
                let offset = direction.to_offset();
                let branch_pos = BlockPos(pos.0.add_raw(offset.x, 0, offset.z));
                let sideways_state =
                    Self::get_sideways_state(trunk_state.id, direction.to_axis()).to_state();
                if TrunkPlacer::place(chunk, &branch_pos, sideways_state) {
                    logs.push(branch_pos);
                }
            }
        }

        (
            vec![TreeNode {
                center: start_pos.up_height(branch_height),
                foliage_radius: 0,
                giant_trunk: false,
            }],
            logs,
        )
    }

    // Vanilla shuffles all six and drops the vertical ones after
    fn shuffled_directions(random: &mut RandomGenerator) -> Vec<BlockDirection> {
        let mut directions = [
            BlockDirection::Down,
            BlockDirection::Up,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ];
        for i in (1..directions.len()).rev() {
            let j = random.next_bounded_i32(i as i32 + 1) as usize;
            directions.swap(i, j);
        }
        directions
            .into_iter()
            .filter(|direction| direction.to_axis() != Axis::Y)
            .collect()
    }

    fn get_sideways_state(id: BlockStateId, axis: Axis) -> BlockStateId {
        let block = id.to_block();
        let mut props = PaleOakWoodLikeProperties::from_state_id(id);
        props.axis = axis;
        props.to_state_id(block)
    }
}
