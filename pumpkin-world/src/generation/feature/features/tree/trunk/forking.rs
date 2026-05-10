use pumpkin_data::BlockState;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use super::TrunkPlacer;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;
use pumpkin_data::BlockDirection;

pub struct ForkingTrunkPlacer;

impl ForkingTrunkPlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
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

        let mut nodes = Vec::new();
        let mut logs = Vec::new();

        let horizontal_directions = BlockDirection::horizontal();
        let lean_direction = horizontal_directions[random.next_bounded_i32(4) as usize];
        let lean_height = height as i32 - random.next_bounded_i32(4) - 1;
        let mut lean_steps = 3 - random.next_bounded_i32(3);

        let mut current_tx = start_pos.0.x;
        let mut current_tz = start_pos.0.z;
        let mut last_y = None;

        for yo in 0..height as i32 {
            let yy = start_pos.0.y + yo;
            if yo >= lean_height && lean_steps > 0 {
                let offset = lean_direction.to_offset();
                current_tx += offset.x;
                current_tz += offset.z;
                lean_steps -= 1;
            }

            let pos = BlockPos::new(current_tx, yy, current_tz);
            if placer.place(chunk, &pos, trunk_state) {
                logs.push(pos);
                last_y = Some(yy + 1);
            }
        }

        if let Some(ey) = last_y {
            nodes.push(TreeNode {
                center: BlockPos::new(current_tx, ey, current_tz),
                foliage_radius: 1,
                giant_trunk: false,
            });
        }

        let mut branch_tx = start_pos.0.x;
        let mut branch_tz = start_pos.0.z;
        let branch_direction = horizontal_directions[random.next_bounded_i32(4) as usize];

        if branch_direction != lean_direction {
            let branch_pos = lean_height - random.next_bounded_i32(2) - 1;
            let mut branch_steps = 1 + random.next_bounded_i32(3);
            let mut branch_last_y = None;

            let mut yo = branch_pos;
            while yo < height as i32 && branch_steps > 0 {
                if yo >= 1 {
                    let yyx = start_pos.0.y + yo;
                    let offset = branch_direction.to_offset();
                    branch_tx += offset.x;
                    branch_tz += offset.z;
                    let pos = BlockPos::new(branch_tx, yyx, branch_tz);
                    if placer.place(chunk, &pos, trunk_state) {
                        logs.push(pos);
                        branch_last_y = Some(yyx + 1);
                    }
                }
                yo += 1;
                branch_steps -= 1;
            }

            if let Some(ey) = branch_last_y {
                nodes.push(TreeNode {
                    center: BlockPos::new(branch_tx, ey, branch_tz),
                    foliage_radius: 0,
                    giant_trunk: false,
                });
            }
        }

        (nodes, logs)
    }
}
