use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{
    BlockProperties, PoweredRailLikeProperties, RailLikeProperties,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use uuid::Uuid;

pub struct MinecartItem;

impl MinecartItem {
    fn item_to_entity(item: &Item) -> &'static EntityType {
        match item.id {
            val if val == Item::MINECART.id => &EntityType::MINECART,
            val if val == Item::TNT_MINECART.id => &EntityType::TNT_MINECART,
            val if val == Item::CHEST_MINECART.id => &EntityType::CHEST_MINECART,
            val if val == Item::HOPPER_MINECART.id => &EntityType::HOPPER_MINECART,
            val if val == Item::FURNACE_MINECART.id => &EntityType::FURNACE_MINECART,
            val if val == Item::COMMAND_BLOCK_MINECART.id => &EntityType::COMMAND_BLOCK_MINECART,
            _ => unreachable!(),
        }
    }
}

impl ItemMetadata for MinecartItem {
    fn ids() -> Box<[u16]> {
        [
            Item::MINECART.id,
            Item::TNT_MINECART.id,
            Item::CHEST_MINECART.id,
            Item::HOPPER_MINECART.id,
            Item::FURNACE_MINECART.id,
            Item::COMMAND_BLOCK_MINECART.id,
        ]
        .into()
    }
}

impl ItemBehaviour for MinecartItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            if !block.has_tag(&tag::Block::MINECRAFT_RAILS) {
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
            let entity_type = Self::item_to_entity(item.item);
            let pos = location.to_f64();
            let entity = Arc::new(Entity::new(
                Uuid::new_v4(),
                world.clone(),
                Vector3::new(pos.x, pos.y + 0.0625 + height, pos.z),
                entity_type,
                false,
            ));
            world.spawn_entity(entity).await;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
