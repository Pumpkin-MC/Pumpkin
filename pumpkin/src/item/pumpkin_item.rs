use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;

pub trait ItemMetadata {
    const ID: u16;
}

#[async_trait]
pub trait PumpkinItem: Send + Sync {
    async fn normal_use(&self, _block: &Item, _player: &Player, _server: &Server) {}
    async fn use_on_block(
        &self,
        _item: &Item,
        _player: &Player,
        _location: BlockPos,
        _block: &Block,
        _server: &Server,
    ) {
    }
}
