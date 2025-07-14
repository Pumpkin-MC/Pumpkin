use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{
    BlockProperties, PoweredRailLikeProperties, RailLikeProperties,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::{Item, item_properties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

pub struct MinecartItem;

impl MinecartItem {
    fn item_to_entity(item: &Item) -> EntityType {
        match item.id {
            val if val == item_properties::MINECART.id => EntityType::MINECART,
            val if val == item_properties::TNT_MINECART.id => EntityType::TNT_MINECART,
            val if val == item_properties::CHEST_MINECART.id => EntityType::CHEST_MINECART,
            val if val == item_properties::HOPPER_MINECART.id => EntityType::HOPPER_MINECART,
            val if val == item_properties::FURNACE_MINECART.id => EntityType::FURNACE_MINECART,
            val if val == item_properties::COMMAND_BLOCK_MINECART.id => {
                EntityType::COMMAND_BLOCK_MINECART
            }
            _ => unreachable!(),
        }
    }
}

impl ItemMetadata for MinecartItem {
    fn ids() -> Box<[u16]> {
        [
            item_properties::MINECART.id,
            item_properties::TNT_MINECART.id,
            item_properties::CHEST_MINECART.id,
            item_properties::HOPPER_MINECART.id,
            item_properties::FURNACE_MINECART.id,
            item_properties::COMMAND_BLOCK_MINECART.id,
        ]
        .into()
    }
}

#[async_trait]
impl PumpkinItem for MinecartItem {
    async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world().await;

        if !block.is_tagged_with_by_tag(&tag::Block::MINECRAFT_RAILS) {
            return;
        }
        let state_id = world.get_block_state_id(&location).await;
        let is_ascending = if PoweredRailLikeProperties::handles_block_id(block.id) {
            PoweredRailLikeProperties::from_state_id(state_id, block)
                .shape
                .is_ascending()
        } else {
            RailLikeProperties::from_state_id(state_id, block)
                .shape
                .is_ascending()
        };
        let height = if is_ascending { 0.5 } else { 0.0 };
        let entity_type = Self::item_to_entity(item);
        let pos = location.to_f64();
        let entity = Arc::new(Entity::new(
            Uuid::new_v4(),
            world.clone(),
            Vector3::new(pos.x, pos.y + 0.0625 + height, pos.z),
            entity_type,
            false,
        ));
        world.spawn_entity(entity).await;
    }
}
