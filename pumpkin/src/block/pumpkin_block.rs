use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use std::sync::Arc;

use super::BlockIsReplacing;

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

    async fn on_entity_collision(
        &self,
        _world: &Arc<World>,
        _entity: &dyn EntityBase,
        _pos: BlockPos,
        _block: Block,
        _state: BlockState,
        _server: &Server,
    ) {
    }

    fn should_drop_items_on_explosion(&self) -> bool {
        true
    }

    async fn explode(&self, _block: &Block, _world: &Arc<World>, _location: BlockPos) {}

    /// Handles the block event, which is an event specific to a block with an integer ID and data.
    ///
    /// returns whether the event was handled successfully
    async fn on_synced_block_event(
        &self,
        _block: &Block,
        _world: &Arc<World>,
        _pos: &BlockPos,
        _type: u8,
        _data: u8,
    ) -> bool {
        false
    }

    #[allow(clippy::too_many_arguments)]
    /// getPlacementState in source code
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        _player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        block.default_state_id
    }

    async fn random_tick(&self, _block: &Block, _world: &Arc<World>, _pos: &BlockPos) {}

    #[allow(clippy::too_many_arguments)]
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        true
    }

    async fn can_update_at(
        &self,
        _world: &World,
        _block: &Block,
        _state_id: BlockStateId,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: &SUseItemOn,
    ) -> bool {
        false
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
        _face: BlockDirection,
        _player: &Player,
    ) {
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Arc<Player>,
        _location: BlockPos,
        _server: &Server,
        _world: Arc<World>,
        _state: BlockState,
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
        _direction: BlockDirection,
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
        _direction: BlockDirection,
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
        _direction: BlockDirection,
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
        _direction: BlockDirection,
    ) -> u8 {
        0
    }
}
