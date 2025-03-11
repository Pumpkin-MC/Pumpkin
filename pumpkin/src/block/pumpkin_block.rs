use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockState, HorizontalFacing};
use pumpkin_data::item::Item;
use pumpkin_inventory::OpenContainer;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::sync::Arc;

pub trait BlockMetadata {
    fn namespace(&self) -> &'static str;
    fn id(&self) -> &'static str;
    fn name(&self) -> String {
        format!("{}:{}", self.namespace(), self.id())
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
        _world: &World,
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
        _world: &World,
    ) -> BlockActionResult {
        BlockActionResult::Continue
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &HorizontalFacing,
        _other: bool,
    ) -> u16 {
        block.default_state_id
    }

    async fn can_place(
        &self,
        _server: &Server,
        _world: &World,
        _block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _player_direction: &HorizontalFacing,
    ) -> bool {
        true
    }

    async fn placed(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
        _world: &World,
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
        _server: &Server,
        _world: &World,
        _block: &Block,
        _block_pos: &BlockPos,
        _source_face: &BlockDirection,
        _source_block_pos: &BlockPos,
    ) {
    }
}
