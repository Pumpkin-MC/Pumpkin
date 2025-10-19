use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::{BlockBehaviour, OnPlaceArgs};
use async_trait::async_trait;
use pumpkin_data::block_properties::AcaciaShelfLikeProperties;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SideChain;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct Shelf;

#[async_trait]
impl BlockBehaviour for Shelf {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = AcaciaShelfLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.powered = block_receives_redstone_power(args.world, args.position).await;
        props.side_chain = SideChain::Unconnected;
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }
}
