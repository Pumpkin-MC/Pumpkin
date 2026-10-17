use pumpkin_data::block_properties::{Facing, MovingPistonLikeProperties, PistonType};
use pumpkin_data::{Block, BlockState, BlockStateId, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::blocks::piston::piston::try_move;
use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnNeighborUpdateArgs, PathComputationType,
};

use super::piston::PistonProps;

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

impl PistonHeadBlock {
    fn is_fitting_base(
        head: PistonHeadProperties,
        base_block: &Block,
        base_state_id: BlockStateId,
    ) -> bool {
        let fitting_block = if head.r#type == PistonType::Normal {
            &Block::PISTON
        } else {
            &Block::STICKY_PISTON
        };
        if base_block != fitting_block {
            return false;
        }
        let base = PistonProps::from_state_id(base_state_id);
        base.extended && base.facing == head.facing
    }

    #[must_use]
    pub fn can_survive(world: &dyn BlockAccessor, pos: &BlockPos, state_id: BlockStateId) -> bool {
        let head = PistonHeadProperties::from_state_id(state_id);
        let base_pos = pos.offset(head.facing.opposite().to_block_direction().to_offset());
        let (base_block, base_state) = world.get_block_and_state(&base_pos);
        Self::is_fitting_base(head, base_block, base_state.id)
            || (base_block == &Block::MOVING_PISTON
                && MovingPistonLikeProperties::from_state_id(base_state.id).facing == head.facing)
    }
}

impl BlockBehaviour for PistonHeadBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive(args.block_accessor, args.position, args.state.id)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let facing = PistonHeadProperties::from_state_id(args.state_id).facing;
        if args.direction.opposite() == facing.to_block_direction()
            && !Self::can_survive(args.world, args.position, args.state_id)
        {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

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

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
