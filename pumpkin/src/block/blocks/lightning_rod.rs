use crate::block::{BlockBehaviour, OnPlaceArgs};
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::LightningRodLikeProperties;
use pumpkin_data::block_properties::Facing;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

#[pumpkin_block_from_tag("minecraft:lightning_rods")]
pub struct LightningRodBlock;

#[async_trait]
impl BlockBehaviour for LightningRodBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LightningRodLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();
        props.r#powered = false;
        props.facing = match args.direction {
            BlockDirection::East => Facing::West,
            BlockDirection::West => Facing::East,
            BlockDirection::Up => Facing::Down,
            BlockDirection::Down => Facing::Up,
            BlockDirection::North => Facing::South,
            BlockDirection::South => Facing::North,
        };
        props.to_state_id(args.block)
    }
}
