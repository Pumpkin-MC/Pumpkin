use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, registry::Block};
use pumpkin_world::item::ItemStack;

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("waterlogged")]
pub struct Waterlogged(bool);

#[async_trait]
impl BlockProperty for Waterlogged {
    async fn on_place(
        &self,
        world: &World,
        _block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        let block = world.get_block(block_pos).await.unwrap();
        if block.name == "water" {
            return Self::True().value();
        }
        Self::False().value()
    }
    async fn on_interact(&self, value: String, _block: &Block, item: &ItemStack) -> String {
        if value == Self::True().value() && item.item.id == 941 {
            return Self::False().value();
        }
        if value == Self::False().value() && item.item.id == 942 {
            return Self::True().value();
        }
        value
    }
}
