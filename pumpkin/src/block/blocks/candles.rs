use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, CandleLikeProperties, EnumVariants, Integer1To4},
    entity::EntityPose,
    item::Item,
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};

use crate::{
    block::{
        BlockIsReplacing,
        pumpkin_block::{BlockMetadata, PumpkinBlock},
        registry::BlockActionResult,
    },
    entity::{EntityBase, player::Player},
    server::Server,
    world::World,
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
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _location: &BlockPos,
        _face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        if player.get_entity().pose.load() != EntityPose::Crouching {
            if let BlockIsReplacing::Itself(state_id) = replacing {
                let mut properties = CandleLikeProperties::from_state_id(state_id, block);
                if properties.candles.to_index() < 3 {
                    properties.candles = Integer1To4::from_index(properties.candles.to_index() + 1);
                }
                return properties.to_state_id(block);
            }
        }

        let mut properties = CandleLikeProperties::default(block);
        properties.waterlogged = replacing.water_source();
        properties.to_state_id(block)
    }

    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let state = world.get_block_state(&location).await;
        let mut properties = CandleLikeProperties::from_state_id(state.id, block);

        match item.id {
            id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id)
                && item.id == block.id =>
            {
                if properties.candles.to_index() < 3 {
                    properties.candles = Integer1To4::from_index(properties.candles.to_index() + 1);
                }

                world
                    .set_block_state(
                        &location,
                        properties.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                BlockActionResult::Consume
            }
            _ => {
                if properties.lit {
                    properties.lit = false;
                } else {
                    return BlockActionResult::Continue;
                }

                world
                    .set_block_state(
                        &location,
                        properties.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                BlockActionResult::Consume
            }
        }
    }

    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        let state_id = world.get_block_state_id(&location).await;
        let mut properties = CandleLikeProperties::from_state_id(state_id, block);

        if properties.lit {
            properties.lit = false;
        }

        world
            .set_block_state(
                &location,
                properties.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let (support_block, state) = block_accessor
            .get_block_and_block_state(&block_pos.down())
            .await;
        !support_block.is_waterlogged(state.id) && state.is_center_solid(BlockDirection::Up)
    }

    async fn can_update_at(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: &SUseItemOn,
        player: &Player,
    ) -> bool {
        let b = world.get_block(block_pos).await;
        player.get_entity().pose.load() != EntityPose::Crouching
            && CandleLikeProperties::from_state_id(state_id, block).candles != Integer1To4::L4
            && block.id == b.id // only the same color can update
    }
}
