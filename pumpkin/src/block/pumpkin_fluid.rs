use std::sync::Arc;

use crate::entity::{EntityBase, player::Player};
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::{fluid::Fluid, item::Item};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::{server::Server, world::World};

use super::{BlockIsReplacing, registry::BlockActionResult};

#[async_trait]
pub trait PumpkinFluid: Send + Sync {
    async fn normal_use(
        &self,
        _fluid: &Fluid,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
        _world: &Arc<World>,
    ) {
    }
    async fn use_with_item(
        &self,
        _fluid: &Fluid,
        _player: &Player,
        _location: BlockPos,
        _item: &Item,
        _server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
        BlockActionResult::Pass
    }

    async fn placed(
        &self,
        _world: &Arc<World>,
        _fluid: &Fluid,
        _state_id: BlockStateId,
        _block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_place(
        &self,
        _server: &Server,
        _world: &Arc<World>,
        fluid: &Fluid,
        _face: BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _replacing: BlockIsReplacing,
    ) -> BlockStateId {
        fluid.default_state_index
    }

    async fn get_state_for_neighbour_update(
        &self,
        _world: &Arc<World>,
        _fluid: &Fluid,
        _block_pos: &BlockPos,
        _notify: bool,
    ) -> BlockStateId {
        0
    }

    async fn on_neighbor_update(
        &self,
        _world: &Arc<World>,
        _fluid: &Fluid,
        _block_pos: &BlockPos,
        _notify: bool,
    ) {
    }

    async fn on_entity_collision(&self, _entity: &dyn EntityBase) {}

    async fn on_scheduled_tick(&self, _world: &Arc<World>, _fluid: &Fluid, _block_pos: &BlockPos) {}

    async fn random_tick(&self, _fluid: &Fluid, _world: &Arc<World>, _block_pos: &BlockPos) {}

    async fn create_legacy_block(&self, _world: &Arc<World>, _block_pos: &BlockPos) {}
}
