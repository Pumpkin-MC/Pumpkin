pub mod flowing_trait;
pub mod lava;
pub mod pathfinder;
pub mod physics;
pub mod water;

// Re-export for backward compatibility
pub mod flowing {
    pub use super::flowing_trait::*;
    pub use super::pathfinder::*;
    pub use super::physics::*;
}

use super::{BlockIsReplacing, registry::BlockActionResult};
use crate::entity::{EntityBase, player::Player};
use crate::{server::Server, world::World};
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::{fluid::Fluid, item::Item};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub trait FluidBehaviour: Send + Sync {
    fn normal_use(
        &self,
        _fluid: &Fluid,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
        _world: &Arc<World>,
    ) {
    }

    fn use_with_item(
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

    fn placed(
        &self,
        _world: &Arc<World>,
        _fluid: &Fluid,
        _state_id: BlockStateId,
        _block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
    }

    #[expect(clippy::too_many_arguments)]
    fn on_place(
        &self,
        _server: &Server,
        _world: &Arc<World>,
        fluid: &Fluid,
        _face: BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _replacing: BlockIsReplacing,
    ) -> BlockStateId {
        fluid.states[fluid.default_state_index as usize].block_state_id
    }

    fn get_state_for_neighbour_update(
        &self,
        _world: &Arc<World>,
        _fluid: &Fluid,
        _block_pos: &BlockPos,
        _notify: bool,
    ) -> BlockStateId {
        BlockStateId::AIR
    }

    fn on_neighbor_update(
        &self,
        _world: &Arc<World>,
        _fluid: &Fluid,
        _block_pos: &BlockPos,
        _notify: bool,
    ) {
    }

    fn on_entity_collision(&self, _entity: &dyn EntityBase) {}

    fn on_scheduled_tick(&self, _world: &Arc<World>, _fluid: &Fluid, _block_pos: &BlockPos) {}

    fn random_tick(&self, _fluid: &Fluid, _world: &Arc<World>, _block_pos: &BlockPos) {}

    fn create_legacy_block(&self, _world: &Arc<World>, _block_pos: &BlockPos) {}
}
