use pumpkin_data::block_properties::{Facing, MovingPistonLikeProperties};
use pumpkin_data::{Block, BlockState, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::piston::piston::try_move;
use crate::block::{BlockBehaviour, OnNeighborUpdateArgs, PathComputationType};

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

impl BlockBehaviour for PistonHeadBlock {
    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let head_state_id = args.world.get_block_state_id(args.position);
        let head_props = PistonHeadProperties::from_state_id(head_state_id);
        let piston_pos = args.position.offset(
            head_props
                .facing
                .opposite()
                .to_block_direction()
                .to_offset(),
        );
        let piston_block = args.world.get_block(&piston_pos);
        if &Block::PISTON == piston_block || &Block::STICKY_PISTON == piston_block {
            if head_props.facing != Facing::Up {
                return;
            }
            let up_pos = args
                .position
                .offset(head_props.facing.to_block_direction().to_offset());
            let upper_block = args.world.get_block(&up_pos);
            if upper_block != &Block::REDSTONE_BLOCK {
                //Then somebody probably broke the redstone block, try to check if piston should still be extended.
                try_move(args.world, piston_block, &piston_pos);
            }
        } else if !(&Block::MOVING_PISTON == piston_block
            && MovingPistonLikeProperties::from_state_id(
                args.world.get_block_state_id(&piston_pos),
            )
            .facing
                == head_props.facing)
            && args
                .world
                .break_block(args.position, None, BlockFlags::SKIP_DROPS)
                .is_some()
        {
            args.world.set_block_state(
                args.position,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
