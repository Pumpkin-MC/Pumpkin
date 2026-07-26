use pumpkin_data::block_properties::{BlockProperties, MovingPistonLikeProperties, PistonType};
use pumpkin_data::{Block, BlockStateId, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::piston::piston::try_move;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, GetStateForNeighborUpdateArgs, OnNeighborUpdateArgs,
};

use super::piston::PistonProps;

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

fn is_fitting_base(
    head_props: PistonHeadProperties,
    base_block: &Block,
    base_state_id: BlockStateId,
) -> bool {
    let expected_block = match head_props.r#type {
        PistonType::Normal => &Block::PISTON,
        PistonType::Sticky => &Block::STICKY_PISTON,
    };

    if base_block != expected_block {
        return false;
    }

    let base_props = PistonProps::from_state_id(base_state_id, base_block);
    base_props.extended && base_props.facing == head_props.facing
}

fn can_survive(
    head_props: PistonHeadProperties,
    world: &crate::world::World,
    pos: &BlockPos,
) -> bool {
    let base_pos = pos.offset(
        head_props
            .facing
            .opposite()
            .to_block_direction()
            .to_offset(),
    );
    let (base_block, base_state_id) = world.get_block_and_state_id(&base_pos);

    is_fitting_base(head_props, base_block, base_state_id)
        || (base_block == &Block::MOVING_PISTON
            && MovingPistonLikeProperties::from_state_id(base_state_id, base_block).facing
                == head_props.facing)
}

impl BlockBehaviour for PistonHeadBlock {
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let props = PistonHeadProperties::from_state_id(args.state.id, &Block::PISTON_HEAD);
            let pos = args
                .position
                .offset(props.facing.opposite().to_block_direction().to_offset());
            let (new_block, new_state) = args.world.get_block_and_state_id(&pos);
            if is_fitting_base(props, new_block, new_state) {
                args.world
                    .break_block(&pos, None, BlockFlags::SKIP_DROPS)
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let head_props = PistonHeadProperties::from_state_id(args.state_id, args.block);
            if args.direction.opposite() == head_props.facing.to_block_direction()
                && !can_survive(head_props, args.world, args.position)
            {
                BlockStateId::AIR
            } else {
                args.state_id
            }
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let head_state_id = args.world.get_block_state_id(args.position);
            if Block::from_state_id(head_state_id) != &Block::PISTON_HEAD {
                return;
            }
            let head_props =
                PistonHeadProperties::from_state_id(head_state_id, &Block::PISTON_HEAD);
            let piston_pos = args.position.offset(
                head_props
                    .facing
                    .opposite()
                    .to_block_direction()
                    .to_offset(),
            );
            let (piston_block, piston_state_id) = args.world.get_block_and_state_id(&piston_pos);
            if is_fitting_base(head_props, piston_block, piston_state_id) {
                try_move(args.world, piston_block, &piston_pos).await;
            }
        })
    }
}
