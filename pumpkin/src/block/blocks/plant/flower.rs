use async_trait::async_trait;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use std::sync::Arc;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

pub struct FlowerBlock;

impl BlockMetadata for FlowerBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:flowers/small").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for FlowerBlock {
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
        <Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &Arc<World>,
        _block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(self, world.as_ref(), pos, state)
            .await
    }

    async fn random_tick(&self, block: &Block, world: &Arc<World>, pos: &BlockPos) {
        //TODO add trail particule
        if world.dimension_type.eq(&VanillaDimensionType::Overworld)
            || world
                .dimension_type
                .eq(&VanillaDimensionType::OverworldCaves)
        {
            if block.eq(&Block::CLOSED_EYEBLOSSOM)
                && world.level_time.lock().await.time_of_day > 14500
            {
                world
                    .set_block_state(
                        pos,
                        Block::OPEN_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            } else if block.eq(&Block::OPEN_EYEBLOSSOM)
                && world.level_time.lock().await.time_of_day <= 14500
            {
                world
                    .set_block_state(
                        pos,
                        Block::CLOSED_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
    }
}

impl PlantBlockBase for FlowerBlock {}
