use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_inventory::OpenContainer;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::registry::Block;

use super::properties::Direction;

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

    #[allow(clippy::too_many_arguments)]
    async fn on_place(
        &self,
        server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        other: bool,
    ) -> u16 {
        server
            .block_properties_manager
            .on_place_state(
                world,
                block,
                face,
                block_pos,
                use_item_on,
                player_direction,
                other,
            )
            .await
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
