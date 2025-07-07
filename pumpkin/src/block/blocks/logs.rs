use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_world::BlockStateId;

use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};

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
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut log_props = LogProperties::default(args.block);
        log_props.axis = args.direction.to_axis();

        log_props.to_state_id(args.block)
    }
}
