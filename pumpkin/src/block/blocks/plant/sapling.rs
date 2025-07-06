use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, Integer0To1};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use std::sync::Arc;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::tag::{RegistryKey, Tagable, get_tag_values};

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

type SaplingProperties = pumpkin_data::block_properties::OakSaplingLikeProperties;

pub struct SaplingBlock;

impl SaplingBlock {
    async fn generate(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_block_state(pos).await;
        let mut props = SaplingProperties::from_state_id(state.id, block);
        if props.stage == Integer0To1::L0 {
            props.stage = Integer0To1::L1;
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
        } else {
            //TODO generate tree
        }
    }
}

impl BlockMetadata for SaplingBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:saplings").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SaplingBlock {
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

    async fn random_tick(&self, _block: &Block, world: &Arc<World>, pos: &BlockPos) {
        self.generate(world, pos).await;
    }
}

impl PlantBlockBase for SaplingBlock {}
