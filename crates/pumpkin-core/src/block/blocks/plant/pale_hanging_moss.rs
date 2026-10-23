use pumpkin_data::{
    Block, BlockDirection, BlockStateId, block_properties::PaleHangingMossLikeProperties,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};

#[pumpkin_block("minecraft:pale_hanging_moss")]
pub struct PaleHangingMossBlock;

impl PaleHangingMossBlock {
    #[must_use]
    pub fn can_stay_at_position(accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let above_pos = pos.up();
        let (above_block, above_state) = accessor.get_block_and_state(&above_pos);
        (above_state.is_side_solid(BlockDirection::Down) && above_block.is_solid())
            || above_block == &Block::PALE_HANGING_MOSS
    }

    #[must_use]
    pub fn get_tip(accessor: &dyn BlockAccessor, pos: &BlockPos) -> BlockPos {
        let mut cur = *pos;
        loop {
            let next = cur.down();
            if accessor.get_block(&next) == &Block::PALE_HANGING_MOSS {
                cur = next;
            } else {
                return cur;
            }
        }
    }

    #[must_use]
    pub fn can_grow_into(accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        accessor.get_block_state(pos).is_air()
    }
}

impl BlockBehaviour for PaleHangingMossBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_stay_at_position(args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = PaleHangingMossLikeProperties::default(args.block);
        props.tip = args.world.get_block(&args.position.down()) != &Block::PALE_HANGING_MOSS;
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::can_stay_at_position(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        let mut props = PaleHangingMossLikeProperties::from_state_id(args.state_id);
        props.tip = args.world.get_block(&args.position.down()) != &Block::PALE_HANGING_MOSS;
        props.to_state_id(args.block)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !Self::can_stay_at_position(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let tip = Self::get_tip(args.world.as_ref(), args.position);
        let below_tip = tip.down();
        args.world.is_in_build_limit(below_tip)
            && Self::can_grow_into(args.world.as_ref(), &below_tip)
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let tip = Self::get_tip(args.world.as_ref(), args.position);
        let below_tip = tip.down();
        if args.world.is_in_build_limit(below_tip)
            && Self::can_grow_into(args.world.as_ref(), &below_tip)
        {
            let mut props = PaleHangingMossLikeProperties::default(&Block::PALE_HANGING_MOSS);
            props.tip = true;
            args.world.set_block_state(
                &below_tip,
                props.to_state_id(&Block::PALE_HANGING_MOSS),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}
