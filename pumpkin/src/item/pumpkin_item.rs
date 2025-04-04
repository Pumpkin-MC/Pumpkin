use crate::entity::Entity;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

pub trait ItemMetadata {
    fn ids() -> Box<[u16]>;
}

#[async_trait]
pub trait PumpkinItem: Send + Sync {
    async fn normal_use(&self, _block: &Item, _player: &Player) {}
    async fn on_attack(&self, _item: &Item, _victim: &Entity) {}

    async fn on_after_dig(
        &self,
        _item: &Item,
        _player: &Player,
        _location: BlockPos,
        _block: &Block,
        _server: &Server,
    ) {
    }

    async fn use_on_block(
        &self,
        _item: &Item,
        _player: &Player,
        _location: BlockPos,
        _face: &BlockDirection,
        _block: &Block,
        _server: &Server,
    ) {
    }

    async fn increment_damage(&self, _player: &Player) {
        let mut inventory = _player.inventory().lock().await;

        // This is considering that the method held_item_mut
        // will return the item that was just used. I believe
        // only items that are not tools (like armor and elytra)
        // can 'be used' without being in hand.
        if let Some(held) = inventory.held_item_mut() {
            held.damage_item();
        }
    }

    fn can_mine(&self, _player: &Player) -> bool {
        true
    }
}
