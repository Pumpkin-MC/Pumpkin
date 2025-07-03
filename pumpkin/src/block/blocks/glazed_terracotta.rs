use crate::block::pumpkin_block::{BlockMetadata, OnPlaceArgs, PumpkinBlock};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_world::BlockStateId;

pub struct GlazedTerracottaBlock;
impl BlockMetadata for GlazedTerracottaBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:glazed_terracottas").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for GlazedTerracottaBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut prop = WallTorchLikeProperties::default(args.block);
        prop.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        prop.to_state_id(args.block)
    }
}
