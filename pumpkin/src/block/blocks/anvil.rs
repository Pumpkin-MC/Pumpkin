use crate::block::pumpkin_block::{BlockMetadata, OnPlaceArgs, PumpkinBlock};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_world::BlockStateId;

pub struct AnvilBlock;

impl BlockMetadata for AnvilBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:anvil").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for AnvilBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let dir = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .rotate_clockwise();

        let mut props = WallTorchLikeProperties::default(args.block);

        props.facing = dir;
        props.to_state_id(args.block)
    }
}
