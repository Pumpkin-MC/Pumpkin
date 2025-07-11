use crate::block::pumpkin_block::{BlockMetadata, OnPlaceArgs, PumpkinBlock};
use crate::entity::EntityBase;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, WhiteBannerLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_world::BlockStateId;

pub struct BannerBlock;

impl BlockMetadata for BannerBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:banners").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for BannerBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = WhiteBannerLikeProperties::default(args.block);
        props.rotation = args.player.get_entity().get_flipped_rotation_16();
        props.to_state_id(args.block)
    }
}
