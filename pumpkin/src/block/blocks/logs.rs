use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

use crate::block::OnPlaceArgs;
use crate::block::{BlockBehaviour, BlockFuture};

type LogProperties = pumpkin_data::block_properties::PaleOakWoodLikeProperties;

#[pumpkin_block_from_tag("minecraft:logs")]
pub struct LogBlock;

impl BlockBehaviour for LogBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut log_props = LogProperties::default(args.block);
            log_props.axis = args.direction.to_axis();

            log_props.to_state_id(args.block)
        })
    }
}
