use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::{BlockFlags, World};
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockState, HorizontalFacing};
use pumpkin_data::item::Item;
use pumpkin_inventory::OpenContainer;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

pub trait BlockMetadata {
    fn namespace(&self) -> &'static str;
    fn ids(&self) -> &'static [&'static str];
    fn names(&self) -> Vec<String> {
        self.ids()
            .iter()
            .map(|f| format!("{}:{}", self.namespace(), f))
            .collect()
    }
}

#[async_trait]
pub trait PumpkinBlock: Send + Sync {
    async fn normal_use(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
        _world: &Arc<World>,
    ) {
    }
    fn should_drop_items_on_explosion(&self) -> bool {
        true
    }
    async fn explode(&self, _block: &Block, _world: &Arc<World>, _location: BlockPos) {}
    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _item: &Item,
        _server: &Server,
        _world: &Arc<World>,
    ) -> BlockActionResult {
        BlockActionResult::Continue
    }

    #[allow(clippy::too_many_arguments)]
    /// getPlacementState in source code
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        _face: &BlockDirection,
        _pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &HorizontalFacing,
        _other: bool,
    ) -> BlockStateId {
        block.default_state_id
    }

    async fn random_tick(&self, _block: &Block, _world: &Arc<World>, _pos: &BlockPos) {}

    async fn can_place_at(&self, _world: &World, _pos: &BlockPos) -> bool {
        true
    }

    /// onBlockAdded in source code
    async fn placed(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        _pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
    }

    async fn player_placed(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _state_id: u16,
        _pos: &BlockPos,
        _face: &BlockDirection,
        _player: &Player,
    ) {
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
        _world: Arc<World>,
        _state: BlockState,
    ) {
    }

    async fn close(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
        _container: &mut OpenContainer,
    ) {
    }

    async fn on_neighbor_update(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
    }

    /// Called if a block state is replaced or it replaces another state
    async fn prepare(
        &self,
        _world: &Arc<World>,
        _pos: &BlockPos,
        _block: &Block,
        _state_id: BlockStateId,
        _flags: BlockFlags,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_state_for_neighbor_update(
        &self,
        _world: &World,
        _block: &Block,
        state: BlockStateId,
        _pos: &BlockPos,
        _direction: &BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        state
    }

    async fn on_scheduled_tick(&self, _world: &Arc<World>, _block: &Block, _pos: &BlockPos) {}

    async fn on_state_replaced(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _location: BlockPos,
        _old_state_id: BlockStateId,
        _moved: bool,
    ) {
    }

    /// Sides where redstone connects to
    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: &BlockDirection,
    ) -> bool {
        false
    }

    /// Weak redstone power, aka. block that should be powered needs to be directly next to the source block
    async fn get_weak_redstone_power(
        &self,
        _block: &Block,
        _world: &World,
        _pos: &BlockPos,
        _state: &BlockState,
        _direction: &BlockDirection,
    ) -> u8 {
        0
    }

    /// Strong redstone power. this can power a block that then gives power
    async fn get_strong_redstone_power(
        &self,
        _block: &Block,
        _world: &World,
        _pos: &BlockPos,
        _state: &BlockState,
        _direction: &BlockDirection,
    ) -> u8 {
        0
    }
}
