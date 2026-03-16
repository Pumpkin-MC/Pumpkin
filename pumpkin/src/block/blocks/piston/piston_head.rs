use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::{Block, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::piston::piston::try_move;
use crate::block::{BlockBehaviour, BlockFuture};
use crate::block::{BrokenArgs, OnNeighborUpdateArgs};

use super::piston::PistonProps;

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

impl BlockBehaviour for PistonHeadBlock {
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = PistonHeadProperties::from_state_id(args.state.id, &Block::PISTON_HEAD);
            let pos = args
                .position
                .offset(props.facing.opposite().to_block_direction().to_offset());
            let (new_block, new_state) = args.world.get_block_and_state_id(&pos).await;
            if &Block::PISTON == new_block || &Block::STICKY_PISTON == new_block {
                let props = PistonProps::from_state_id(new_state, new_block);
                if props.extended {
                    // TODO: use player
                    args.world
                        .break_block(&pos, None, BlockFlags::SKIP_DROPS)
                        .await;
                }
            }
        })
    }
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let head_state_id = args.world.get_block_state_id(args.position).await;
            let head_props =
                PistonHeadProperties::from_state_id(head_state_id, &Block::PISTON_HEAD);
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
            let (piston_block, _) = args.world.get_block_and_state_id(&piston_pos).await;
            if &Block::PISTON == piston_block || &Block::STICKY_PISTON == piston_block {
                let up_pos = args
                    .position
                    .offset(head_props.facing.to_block_direction().to_offset());
                let (upper_block, _) = args.world.get_block_and_state_id(&up_pos).await;
                if upper_block != &Block::REDSTONE_BLOCK {
                    //Then somebody probably broke the redstone block, try to check if piston should still be extended.
                    try_move(args.world, piston_block, &piston_pos).await;
                }
            }
        })
    }
}
