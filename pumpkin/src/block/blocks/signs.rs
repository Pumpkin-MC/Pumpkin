use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::entities::sign::SignBlockEntity;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

type SignProperties = pumpkin_data::block_properties::OakSignLikeProperties;

pub struct SignBlock;

impl BlockMetadata for SignBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:signs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SignBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        _player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> u16 {
        let mut sign_props = SignProperties::default(block);
        sign_props.waterlogged = replacing.water_source();

        sign_props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: u16,
        pos: &BlockPos,
        _old_state_id: u16,
        _notify: bool,
    ) {
        world
            .add_block_entity(Arc::new(SignBlockEntity::empty(*pos)))
            .await;
    }

    async fn player_placed(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _state_id: u16,
        pos: &BlockPos,
        _face: BlockDirection,
        player: &Player,
    ) {
        player.send_sign_packet(*pos).await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: u16,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }
}
