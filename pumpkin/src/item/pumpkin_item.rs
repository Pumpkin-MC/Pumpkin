use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;

pub trait ItemMetadata {
    const IDS: &'static [u16];
}

#[async_trait]
pub trait PumpkinItem: Send + Sync {
    async fn normal_use(&self, _block: &Item, _player: &Player) {}
    async fn use_on_block(
        &self,
        _item: &Item,
        _player: &Player,
        _location: BlockPos,
        _block: &Block,
        _server: &Server,
    ) {
    }

    fn can_mine(&self, _player: &Player) -> bool {
        true
    }
}
