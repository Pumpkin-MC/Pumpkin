use crate::block::{BlockBehaviour, OnPlaceArgs};
use crate::entity::EntityBase;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, WhiteBannerLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

#[pumpkin_block_from_tag("minecraft:banners")]
pub struct BannerBlock;

#[async_trait]
impl BlockBehaviour for BannerBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = WhiteBannerLikeProperties::default(args.block);
        props.rotation = args.player.get_entity().get_flipped_rotation_16();
        props.to_state_id(args.block)
    }
}
