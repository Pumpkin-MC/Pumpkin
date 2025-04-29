use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::World;
use crate::{entity::player::Player, server::Server};

type LogProperties = pumpkin_data::block_properties::PaleOakWoodLikeProperties;

pub struct LogBlock;
impl BlockMetadata for LogBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:logs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for LogBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player: &Player,
        _replacing: BlockIsReplacing,
    ) -> BlockStateId {
        let mut log_props = LogProperties::default(block);
        log_props.axis = face.to_axis();

        log_props.to_state_id(block)
    }
}
