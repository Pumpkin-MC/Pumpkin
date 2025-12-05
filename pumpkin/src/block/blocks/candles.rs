use pumpkin_data::item::Item;
use pumpkin_data::{
    BlockDirection,
    block_properties::{BlockProperties, CandleLikeProperties, EnumVariants, Integer1To4},
    entity::EntityPose,
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::block::BlockFuture;
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
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.player.get_entity().pose.load() != EntityPose::Crouching
                && let BlockIsReplacing::Itself(state_id) = args.replacing
            {
                let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);
                if properties.candles.to_index() < 3 {
                    properties.candles = Integer1To4::from_index(properties.candles.to_index() + 1);
                }
                return properties.to_state_id(args.block);
            }

            let mut properties = CandleLikeProperties::default(args.block);
            properties.waterlogged = args.replacing.water_source();
            properties.to_state_id(args.block)
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut properties = CandleLikeProperties::from_state_id(state.id, args.block);

            let item_lock = args.item_stack.lock().await;
            let item = item_lock.item;
            drop(item_lock);
            match item.id {
                id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id)
                    && item.id == args.block.id =>
                {
                    if properties.candles.to_index() < 3 {
                        properties.candles =
                            Integer1To4::from_index(properties.candles.to_index() + 1);
                    }

                    args.world
                        .set_block_state(
                            args.position,
                            properties.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    BlockActionResult::Consume
                }
                _ => {
                    if properties.lit {
                        properties.lit = false;
                    } else {
                        return BlockActionResult::Pass;
                    }

                    args.world
                        .set_block_state(
                            args.position,
                            properties.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    BlockActionResult::Consume
                }
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position).await;
            let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);

            if properties.lit {
                properties.lit = false;
            }

            args.world
                .set_block_state(
                    args.position,
                    properties.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            BlockActionResult::Consume
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let (support_block, state) = args
                .block_accessor
                .get_block_and_state(&args.position.down())
                .await;
            !support_block.is_waterlogged(state.id) && state.is_center_solid(BlockDirection::Up)
        })
    }

    fn can_update_at<'a>(&'a self, args: CanUpdateAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let b = args.world.get_block(args.position).await;
            args.player.get_entity().pose.load() != EntityPose::Crouching
                && CandleLikeProperties::from_state_id(args.state_id, args.block).candles
                    != Integer1To4::L4
                && args.block.id == b.id // only the same color can update
        })
    }
}
