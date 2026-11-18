use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::vehicle::boat::BoatEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;

pub struct BoatItem;

impl BoatItem {
    /// Maps boat item to corresponding entity type
    pub(crate) fn item_to_entity(item: &Item) -> &'static EntityType {
        match item.id {
            val if val == Item::OAK_BOAT.id => &EntityType::OAK_BOAT,
            val if val == Item::OAK_CHEST_BOAT.id => &EntityType::OAK_CHEST_BOAT,
            val if val == Item::SPRUCE_BOAT.id => &EntityType::SPRUCE_BOAT,
            val if val == Item::SPRUCE_CHEST_BOAT.id => &EntityType::SPRUCE_CHEST_BOAT,
            val if val == Item::BIRCH_BOAT.id => &EntityType::BIRCH_BOAT,
            val if val == Item::BIRCH_CHEST_BOAT.id => &EntityType::BIRCH_CHEST_BOAT,
            val if val == Item::JUNGLE_BOAT.id => &EntityType::JUNGLE_BOAT,
            val if val == Item::JUNGLE_CHEST_BOAT.id => &EntityType::JUNGLE_CHEST_BOAT,
            val if val == Item::ACACIA_BOAT.id => &EntityType::ACACIA_BOAT,
            val if val == Item::ACACIA_CHEST_BOAT.id => &EntityType::ACACIA_CHEST_BOAT,
            val if val == Item::DARK_OAK_BOAT.id => &EntityType::DARK_OAK_BOAT,
            val if val == Item::DARK_OAK_CHEST_BOAT.id => &EntityType::DARK_OAK_CHEST_BOAT,
            val if val == Item::MANGROVE_BOAT.id => &EntityType::MANGROVE_BOAT,
            val if val == Item::MANGROVE_CHEST_BOAT.id => &EntityType::MANGROVE_CHEST_BOAT,
            val if val == Item::CHERRY_BOAT.id => &EntityType::CHERRY_BOAT,
            val if val == Item::CHERRY_CHEST_BOAT.id => &EntityType::CHERRY_CHEST_BOAT,
            val if val == Item::PALE_OAK_BOAT.id => &EntityType::PALE_OAK_BOAT,
            val if val == Item::PALE_OAK_CHEST_BOAT.id => &EntityType::PALE_OAK_CHEST_BOAT,
            val if val == Item::POPLAR_BOAT.id => &EntityType::POPLAR_BOAT,
            val if val == Item::POPLAR_CHEST_BOAT.id => &EntityType::POPLAR_CHEST_BOAT,
            val if val == Item::BAMBOO_RAFT.id => &EntityType::BAMBOO_RAFT,
            val if val == Item::BAMBOO_CHEST_RAFT.id => &EntityType::BAMBOO_CHEST_RAFT,
            _ => {
                tracing::error!("Unknown boat item ID: {}", item.id);
                &EntityType::OAK_BOAT
            }
        }
    }
}

impl ItemMetadata for BoatItem {
    fn ids() -> Box<[u16]> {
        [
            Item::OAK_BOAT.id,
            Item::OAK_CHEST_BOAT.id,
            Item::SPRUCE_BOAT.id,
            Item::SPRUCE_CHEST_BOAT.id,
            Item::BIRCH_BOAT.id,
            Item::BIRCH_CHEST_BOAT.id,
            Item::JUNGLE_BOAT.id,
            Item::JUNGLE_CHEST_BOAT.id,
            Item::ACACIA_BOAT.id,
            Item::ACACIA_CHEST_BOAT.id,
            Item::DARK_OAK_BOAT.id,
            Item::DARK_OAK_CHEST_BOAT.id,
            Item::MANGROVE_BOAT.id,
            Item::MANGROVE_CHEST_BOAT.id,
            Item::CHERRY_BOAT.id,
            Item::CHERRY_CHEST_BOAT.id,
            Item::PALE_OAK_BOAT.id,
            Item::PALE_OAK_CHEST_BOAT.id,
            Item::POPLAR_BOAT.id,
            Item::POPLAR_CHEST_BOAT.id,
            Item::BAMBOO_RAFT.id,
            Item::BAMBOO_CHEST_RAFT.id,
        ]
        .into()
    }
}

impl ItemBehaviour for BoatItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, Hand::Right);
    }

    /// Vanilla: `BoatItem.use()` - raycasts to find placement position
    fn normal_use_with_hand(&self, item: &Item, player: &Player, yaw: f32, pitch: f32, hand: Hand) {
        let world = player.world();
        let (start_pos, end_pos) = self.get_start_and_end_pos_with_rotation(player, yaw, pitch);

        // Vanilla: BoatItem.use() clips with FluidHandling.ANY and Block.OUTLINE
        let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
            start_pos,
            end_pos,
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::Any,
        ));
        if hit.is_miss {
            return;
        }

        // Vanilla rejects placement when the player's eye is inside a pickable entity.
        let rotation_vec = Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)) * 5.0;
        let player_entity = player.get_entity().bounding_box.load();
        let search_box = player_entity.stretch(rotation_vec).expand_all(1.0);
        if world.get_all_at_box(&search_box).iter().any(|entity| {
            let target = entity.get_entity();
            target.entity_id != player.entity_id()
                && !entity.is_spectator()
                && entity.can_hit()
                && target.bounding_box.load().contains(start_pos)
        }) {
            return;
        }

        let mut stack = player.inventory.get_stack_in_hand(hand);
        let hit_vec = hit.location;
        let entity = Entity::new(world.clone(), hit_vec, Self::item_to_entity(item));
        entity.set_rotation(yaw, 0.0);
        let boat_entity = Arc::new(BoatEntity::new(entity));
        crate::item::items::spawn_egg::apply_entity_variant(
            &stack,
            boat_entity.as_ref(),
            Some(player),
        );
        let entity = boat_entity.get_entity();
        let boat_box = entity.bounding_box.load();
        if !world
            .get_block_collisions(boat_box, boat_entity.as_ref())
            .0
            .is_empty()
            || !world.no_border_collision(boat_entity.as_ref(), boat_box)
            || entity
                .get_boat_collisions(boat_box, boat_entity.as_ref())
                .iter()
                .any(|shape| shape.intersects(&boat_box))
        {
            return;
        }

        world.spawn_entity(boat_entity);
        world.emit_game_event(GameEvent::EntityPlace.name(), hit_vec);

        if !stack.is_empty() && stack.item.id == item.id {
            stack.decrement_unless_creative(player.gamemode.load(), 1);
            player.inventory.set_stack_in_hand(hand, stack.clone());
            let slot_index = match hand {
                Hand::Right => player.inventory.get_selected_slot() as usize,
                Hand::Left => PlayerInventory::OFF_HAND_SLOT,
            };
            player.sync_hand_slot(slot_index, stack);
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
