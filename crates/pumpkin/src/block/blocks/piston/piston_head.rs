use pumpkin_data::block_properties::Facing;
use pumpkin_data::{Block, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::blocks::piston::piston::try_move;
use crate::block::{BrokenArgs, OnNeighborUpdateArgs};

use super::piston::PistonProps;

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

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
                // TODO: use player
                args.world.break_block(&pos, None, BlockFlags::SKIP_DROPS);
            }
        }
    }
    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let head_state_id = args.world.get_block_state_id(args.position);
        let head_props = PistonHeadProperties::from_state_id(head_state_id);
        if head_props.facing != Facing::Up {
            return;
        }
        let piston_pos = args.position.offset(
            head_props
                .facing
                .opposite()
                .to_block_direction()
                .to_offset(),
        );
        let piston_block = args.world.get_block(&piston_pos);
        if &Block::PISTON == piston_block || &Block::STICKY_PISTON == piston_block {
            let up_pos = args
                .position
                .offset(head_props.facing.to_block_direction().to_offset());
            let upper_block = args.world.get_block(&up_pos);
            if upper_block != &Block::REDSTONE_BLOCK {
                //Then somebody probably broke the redstone block, try to check if piston should still be extended.
                try_move(args.world, piston_block, &piston_pos);
            }
        }
    }
}
