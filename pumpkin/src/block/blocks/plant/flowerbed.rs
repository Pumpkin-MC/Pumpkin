use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};
use std::sync::Arc;

use crate::block::BlockIsReplacing;
use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

use super::segmented::{PlaceContext, Segmented, UpdateContext};

type FlowerbedProperties = pumpkin_data::block_properties::PinkPetalsLikeProperties;

pub struct FlowerbedBlock;

impl BlockMetadata for FlowerbedBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["pink_petals", "wildflowers"]
    }
}

#[async_trait]
impl PumpkinBlock for FlowerbedBlock {
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
        let block_below = block_accessor.get_block(&block_pos.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == Block::FARMLAND
    }

    async fn can_update_at(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        face: BlockDirection,
        use_item_on: &SUseItemOn,
        player: &Player,
    ) -> bool {
        let ctx = UpdateContext {
            world,
            block,
            state_id,
            block_pos,
            face,
            use_item_on,
            player,
        };
        Segmented::can_update_at(self, &ctx).await
    }

    async fn on_place(
        &self,
        server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        face: BlockDirection,
        replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let ctx = PlaceContext {
            server,
            world,
            player,
            block,
            block_pos,
            face,
            replacing,
            use_item_on,
        };
        Segmented::on_place(self, &ctx).await
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
}

impl PlantBlockBase for FlowerbedBlock {}

impl Segmented for FlowerbedBlock {
    type Properties = FlowerbedProperties;
}
