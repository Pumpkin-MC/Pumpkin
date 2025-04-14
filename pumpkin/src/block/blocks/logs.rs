use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::server::Server;
use crate::world::World;

type LogProperties = pumpkin_data::block::PaleOakWoodLikeProperties;

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
        _player_direction: &HorizontalFacing,
        _other: bool,
    ) -> BlockStateId {
        let mut log_props = LogProperties::default(block);
        log_props.axis = face.to_axis();

        log_props.to_state_id(block)
    }
}
