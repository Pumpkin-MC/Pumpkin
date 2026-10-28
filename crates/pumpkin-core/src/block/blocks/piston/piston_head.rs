use pumpkin_data::block_properties::{MovingPistonLikeProperties, PistonType};
use pumpkin_data::{Block, BlockState, BlockStateId, FacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnNeighborUpdateArgs, OnStateReplacedArgs, PathComputationType,
};

use super::piston::PistonProps;

pub(crate) type PistonHeadProperties = pumpkin_data::block_properties::PistonHeadLikeProperties;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

impl PistonHeadBlock {
    fn is_fitting_base(
        head_props: PistonHeadProperties,
        base_block: &Block,
        base_state_id: BlockStateId,
    ) -> bool {
        let expected_base = if head_props.r#type == PistonType::Normal {
            &Block::PISTON
        } else {
            &Block::STICKY_PISTON
        };
        if base_block != expected_base {
            return false;
        }
        let base_props = PistonProps::from_state_id(base_state_id);
        base_props.extended && base_props.facing == head_props.facing
    }
}

impl BlockBehaviour for PistonHeadBlock {
    fn broken(&self, args: BrokenArgs<'_>) {
        let props = PistonHeadProperties::from_state_id(args.state.id);
        let pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (new_block, new_state) = args.world.get_block_and_state_id(&pos);
        if Self::is_fitting_base(props, new_block, new_state) {
            let flags = if args.player.gamemode.load() == GameMode::Creative {
                BlockFlags::SKIP_DROPS
            } else {
                BlockFlags::empty()
            };
            args.world.break_block(&pos, None, flags);
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved {
            return;
        }
        let props = PistonHeadProperties::from_state_id(args.old_state_id);
        let base_pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (base_block, base_state) = args.world.get_block_and_state_id(&base_pos);
        if Self::is_fitting_base(props, base_block, base_state) {
            args.world.break_block(&base_pos, None, BlockFlags::empty());
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = PistonHeadProperties::from_state_id(args.state_id);
        if args.direction.opposite() == props.facing.to_block_direction()
            && !self.can_place_at(CanPlaceAtArgs {
                server: None,
                world: None,
                block_accessor: args.world,
                block: args.block,
                state: BlockState::from_id(args.state_id),
                position: args.position,
                direction: None,
                player: None,
                use_item_on: None,
            })
        {
            Block::AIR.default_state.id
        } else {
            args.state_id
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let props = PistonHeadProperties::from_state_id(args.state.id);
        let base_pos = args
            .position
            .offset(props.facing.opposite().to_block_direction().to_offset());
        let (base_block, base_state) = args.block_accessor.get_block_and_state(&base_pos);
        Self::is_fitting_base(props, base_block, base_state.id)
            || (base_block == &Block::MOVING_PISTON
                && MovingPistonLikeProperties::from_state_id(base_state.id).facing == props.facing)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        if self.can_place_at(CanPlaceAtArgs {
            server: None,
            world: Some(args.world),
            block_accessor: args.world.as_ref(),
            block: args.block,
            state,
            position: args.position,
            direction: None,
            player: None,
            use_item_on: None,
        }) {
            let head_props = PistonHeadProperties::from_state_id(state.id);
            let base_pos = args.position.offset(
                head_props
                    .facing
                    .opposite()
                    .to_block_direction()
                    .to_offset(),
            );
            args.world.update_neighbor(&base_pos, args.source_block);
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
