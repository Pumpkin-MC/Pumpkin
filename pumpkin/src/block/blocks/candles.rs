use async_trait::async_trait;
use pumpkin_data::{
    BlockDirection,
    block_properties::{BlockProperties, CandleLikeProperties, EnumVariants, Integer1To4},
    entity::EntityPose,
    item::Item,
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::{
    block::{
        BlockIsReplacing,
        pumpkin_block::{
            BlockMetadata, CanPlaceAtArgs, CanUpdateAtArgs, NormalUseArgs, OnPlaceArgs,
            PumpkinBlock, UseWithItemArgs,
        },
        registry::BlockActionResult,
    },
    entity::EntityBase,
};

pub struct CandleBlock;

impl BlockMetadata for CandleBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:candles").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for CandleBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.player.get_entity().pose.load() != EntityPose::Crouching {
            if let BlockIsReplacing::Itself(state_id) = args.replacing {
                let mut properties = CandleLikeProperties::from_state_id(state_id, args.block);
                if properties.candles.to_index() < 3 {
                    properties.candles = Integer1To4::from_index(properties.candles.to_index() + 1);
                }
                return properties.to_state_id(args.block);
            }
        }

        let mut properties = CandleLikeProperties::default(args.block);
        properties.waterlogged = args.replacing.water_source();
        properties.to_state_id(args.block)
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
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
                    properties.candles = Integer1To4::from_index(properties.candles.to_index() + 1);
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
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
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
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let (support_block, state) = args
            .block_accessor
            .get_block_and_block_state(&args.position.down())
            .await;
        !support_block.is_waterlogged(state.id) && state.is_center_solid(BlockDirection::Up)
    }

    async fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let b = args.world.get_block(args.position).await;
        args.player.get_entity().pose.load() != EntityPose::Crouching
            && CandleLikeProperties::from_state_id(args.state_id, args.block).candles
                != Integer1To4::L4
            && args.block.id == b.id // only the same color can update
    }
}
