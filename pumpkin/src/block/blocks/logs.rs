use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::PumpkinBlock;

type LogProperties = pumpkin_data::block_properties::PaleOakWoodLikeProperties;

#[pumpkin_block_from_tag("minecraft:logs")]
pub struct LogBlock;

#[async_trait]
impl PumpkinBlock for LogBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut log_props = LogProperties::default(args.block);
        log_props.axis = args.direction.to_axis();

        log_props.to_state_id(args.block)
    }
}
