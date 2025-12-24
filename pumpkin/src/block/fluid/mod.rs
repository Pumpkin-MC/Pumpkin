pub mod flowing;
pub mod lava;
pub mod water;

use std::sync::Arc;

use crate::block::BlockFuture;
use crate::entity::{EntityBase, player::Player};
use pumpkin_data::BlockDirection;
use pumpkin_data::{fluid::Fluid, item::Item};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::{server::Server, world::World};

use super::{BlockIsReplacing, registry::BlockActionResult};

pub trait FluidBehaviour: Send + Sync {
    fn normal_use<'a>(
        &'a self,
        _fluid: &'a Fluid,
        _player: &'a Player,
        _location: BlockPos,
        _server: &'a Server,
        _world: &'a Arc<World>,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    fn use_with_item<'a>(
        &'a self,
        _fluid: &'a Fluid,
        _player: &'a Player,
        _location: BlockPos,
        _item: &'a Item,
        _server: &'a Server,
        _world: &'a Arc<World>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async { BlockActionResult::Pass })
    }

    fn placed<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _fluid: &'a Fluid,
        _state_id: BlockStateId,
        _block_pos: &'a BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    #[allow(clippy::too_many_arguments)]
    fn on_place<'a>(
        &'a self,
        _server: &'a Server,
        _world: &'a Arc<World>,
        fluid: &'a Fluid,
        _face: BlockDirection,
        _block_pos: &'a BlockPos,
        _use_item_on: &'a SUseItemOn,
        _replacing: BlockIsReplacing,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async { fluid.default_state_index })
    }

    fn get_state_for_neighbour_update<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _fluid: &'a Fluid,
        _block_pos: &'a BlockPos,
        _notify: bool,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async { 0 })
    }

    fn on_neighbor_update<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _fluid: &'a Fluid,
        _block_pos: &'a BlockPos,
        _notify: bool,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    fn on_entity_collision<'a>(&'a self, _entity: &'a dyn EntityBase) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    fn on_scheduled_tick<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _fluid: &'a Fluid,
        _block_pos: &'a BlockPos,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    fn random_tick<'a>(
        &'a self,
        _fluid: &'a Fluid,
        _world: &'a Arc<World>,
        _block_pos: &'a BlockPos,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }

    fn create_legacy_block<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _block_pos: &'a BlockPos,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {})
    }
}
