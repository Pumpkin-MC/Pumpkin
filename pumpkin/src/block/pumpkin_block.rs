use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_inventory::OpenContainer;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use pumpkin_world::item::registry::Item;

pub trait BlockMetadata {
    const NAMESPACE: &'static str;
    const ID: &'static str;
    fn name(&self) -> String {
        format!("{}:{}", Self::NAMESPACE, Self::ID)
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
    ) {
    }
    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _item: &Item,
        _server: &Server,
    ) -> BlockActionResult {
        BlockActionResult::Continue
    }

    async fn placed(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
    ) {
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _server: &Server,
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
}
