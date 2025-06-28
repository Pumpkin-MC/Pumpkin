use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, EnumVariants, Integer0To1, TorchflowerCropLikeProperties,
};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;
use rand::Rng;
use std::sync::Arc;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

type TorchFlowerProperties = TorchflowerCropLikeProperties;

#[pumpkin_block("minecraft:torchflower_crop")]
pub struct TorchFlowerBlock;

#[async_trait]
impl PumpkinBlock for TorchFlowerBlock {
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
        <Self as CropBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
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
        if rand::rng().random_range(0..2) != 0 {
            <Self as CropBlockBase>::random_tick(self, world, pos).await;
        }
    }
}

impl PlantBlockBase for TorchFlowerBlock {}

impl CropBlockBase for TorchFlowerBlock {
    fn max_age(&self) -> i32 {
        2
    }

    fn get_age(&self, state: &pumpkin_data::BlockState, block: &Block) -> i32 {
        let props = TorchFlowerProperties::from_state_id(state.id, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(
        &self,
        block: &Block,
        state: &pumpkin_data::BlockState,
        age: i32,
    ) -> BlockStateId {
        if age == 1 {
            let mut properties = TorchFlowerProperties::from_state_id(state.id, block);
            properties.age = Integer0To1::L1;
            properties.to_state_id(block)
        } else {
            Block::TORCHFLOWER.default_state.id
        }
    }
}
