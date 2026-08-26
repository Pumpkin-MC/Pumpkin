use pumpkin_data::item::Item;
use pumpkin_data::{
    BlockDirection, BlockStateId,
    block_properties::{BlockProperties, CandleLikeProperties},
    entity::EntityPose,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

use crate::block::{GetStateForNeighborUpdateArgs, OnScheduledTickArgs};
use crate::{
    block::{
        BlockIsReplacing,
        registry::BlockActionResult,
        {
            BlockBehaviour, CanPlaceAtArgs, CanUpdateAtArgs, NormalUseArgs, OnPlaceArgs,
            UseWithItemArgs,
        },
    },
    entity::EntityBase,
};

#[pumpkin_block_from_tag("minecraft:candles")]
pub struct CandleBlock;

impl BlockBehaviour for CandleBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.player.get_entity().pose.load() != EntityPose::Crouching
            && let BlockIsReplacing::Itself(state_id) = args.replacing
        {
            let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);
            if properties.candles < 4 {
                properties.candles += 1;
            }
            return properties.to_state_id(args.block);
        }

        let mut properties = CandleLikeProperties::default(args.block);
        properties.waterlogged = args.replacing.water_source();
        properties.to_state_id(args.block)
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        {
            let state = args.world.get_block_state(args.position);
            let mut properties = CandleLikeProperties::from_state_id(state.id, args.block);

            let item = args.item_stack.item;

            match item.id {
                id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id)
                    && item.id == args.block.item_id =>
                {
                    let was_lit = properties.lit;

                    if properties.candles < 4 {
                        properties.candles += 1;
                    }

                    properties.lit = was_lit;

                    args.world.set_block_state(
                        args.position,
                        properties.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );

                    BlockActionResult::Consume
                }
                _ => {
                    if properties.lit {
                        properties.lit = false;
                    } else {
                        return BlockActionResult::Pass;
                    }

                    args.world.set_block_state(
                        args.position,
                        properties.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );

                    BlockActionResult::Consume
                }
            }
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        {
            let state_id = args.world.get_block_state_id(args.position);
            let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);

            if properties.lit {
                properties.lit = false;
            }

            args.world.set_block_state(
                args.position,
                properties.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );

            BlockActionResult::Consume
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let b = BlockAccessor::get_block(args.world, args.position);
        args.player.get_entity().pose.load() != EntityPose::Crouching
            && CandleLikeProperties::from_state_id(args.state_id, args.block).candles != 4
            && args.block.id == b.id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::empty());
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }
}

fn can_place_at(block_accessor: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let (support_block, state) = block_accessor.get_block_and_state(&position.down());
    !support_block.is_waterlogged(state.id) && state.is_center_solid(BlockDirection::Up)
}
