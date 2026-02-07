use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

pub struct BoatItem;

impl BoatItem {
    fn item_to_entity(item: &Item) -> &'static EntityType {
        match item.id {
            val if val == Item::OAK_BOAT.id => &EntityType::OAK_BOAT,
            val if val == Item::SPRUCE_BOAT.id => &EntityType::SPRUCE_BOAT,
            val if val == Item::BIRCH_BOAT.id => &EntityType::BIRCH_BOAT,
            val if val == Item::JUNGLE_BOAT.id => &EntityType::JUNGLE_BOAT,
            val if val == Item::ACACIA_BOAT.id => &EntityType::ACACIA_BOAT,
            val if val == Item::DARK_OAK_BOAT.id => &EntityType::DARK_OAK_BOAT,
            val if val == Item::CHERRY_BOAT.id => &EntityType::CHERRY_BOAT,
            val if val == Item::MANGROVE_BOAT.id => &EntityType::MANGROVE_BOAT,
            val if val == Item::PALE_OAK_BOAT.id => &EntityType::PALE_OAK_BOAT,
            val if val == Item::OAK_CHEST_BOAT.id => &EntityType::OAK_CHEST_BOAT,
            val if val == Item::SPRUCE_CHEST_BOAT.id => &EntityType::SPRUCE_CHEST_BOAT,
            val if val == Item::BIRCH_CHEST_BOAT.id => &EntityType::BIRCH_CHEST_BOAT,
            val if val == Item::JUNGLE_CHEST_BOAT.id => &EntityType::JUNGLE_CHEST_BOAT,
            val if val == Item::ACACIA_CHEST_BOAT.id => &EntityType::ACACIA_CHEST_BOAT,
            val if val == Item::DARK_OAK_CHEST_BOAT.id => &EntityType::DARK_OAK_CHEST_BOAT,
            val if val == Item::CHERRY_CHEST_BOAT.id => &EntityType::CHERRY_CHEST_BOAT,
            val if val == Item::MANGROVE_CHEST_BOAT.id => &EntityType::MANGROVE_CHEST_BOAT,
            val if val == Item::PALE_OAK_CHEST_BOAT.id => &EntityType::PALE_OAK_CHEST_BOAT,
            val if val == Item::BAMBOO_RAFT.id => &EntityType::OAK_BOAT, // TODO: BAMBOO_RAFT entity
            val if val == Item::BAMBOO_CHEST_RAFT.id => &EntityType::OAK_CHEST_BOAT, // TODO: BAMBOO_CHEST_RAFT entity
            _ => &EntityType::OAK_BOAT,
        }
    }
}

impl ItemMetadata for BoatItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_BOATS.1.to_vec().into_boxed_slice()
    }
}

impl ItemBehaviour for BoatItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();

            // Spawn the boat entity at the clicked position
            let entity_type = Self::item_to_entity(item.item);
            let spawn_pos = match face {
                BlockDirection::Up => location.to_f64().add_raw(0.5, 1.0, 0.5),
                _ => location.to_f64().add_raw(0.5, 0.5, 0.5),
            };
            let entity = Arc::new(Entity::new(world.clone(), spawn_pos, entity_type));
            world.spawn_entity(entity).await;

            // Consume item in survival/adventure mode
            let gamemode = player.gamemode.load();
            item.decrement_unless_creative(gamemode, 1);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
