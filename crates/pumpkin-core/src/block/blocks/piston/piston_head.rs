use pumpkin_data::{Block, BlockState, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::piston::piston::try_move;
use crate::block::{BlockBehaviour, BrokenArgs, OnNeighborUpdateArgs, PathComputationType};

use super::piston::PistonProps;

pub type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;
pub(crate) type MovingPistonProps = pumpkin_data::block_properties::MovingPistonLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

impl BlockBehaviour for PistonHeadBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        let props = PistonHeadProperties::from_state_id(args.state.id);
        let pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (new_block, new_state) = args.world.get_block_and_state_id(&pos);
        if &Block::PISTON == new_block || &Block::STICKY_PISTON == new_block {
            let props = PistonProps::from_state_id(new_state);
            if props.extended {
                args.world.break_block(&pos, Some(args.player), BlockFlags::SKIP_DROPS);
            }
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let head_state = args.world.get_block_state(args.position);
        let head_props = PistonHeadProperties::from_state_id(head_state.id);
        let facing_dir = head_props.facing.to_block_direction();
        let base_pos = args
            .position
            .offset(facing_dir.opposite().to_offset());
        let (base_block, base_state) = args.world.get_block_and_state(&base_pos);

        // Check if head can survive (matching jvmc canSurvive)
        let is_expected_base = match head_props.r#type {
            pumpkin_data::block_properties::PistonType::Normal => base_block == &Block::PISTON,
            pumpkin_data::block_properties::PistonType::Sticky => base_block == &Block::STICKY_PISTON,
        };
        let can_survive = if is_expected_base {
            let base_props = PistonProps::from_state_id(base_state.id);
            base_props.extended && base_props.facing == head_props.facing
        } else if base_block == &Block::MOVING_PISTON {
            let moving_props = MovingPistonProps::from_state_id(base_state.id);
            moving_props.facing == head_props.facing
        } else {
            false
        };

        if !can_survive {
            args.world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS);
            return;
        }

        // Forward neighbor update to piston base (matching jvmc neighborChanged)
        args.world.update_neighbors_at(&base_pos, &Block::PISTON_HEAD, None);
        try_move(args.world, base_block, &base_pos);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
