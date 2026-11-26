use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::vehicle::boat::BoatEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::world::World;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::{GameMode, Hand};
use pumpkin_world::clip::{BlockHitResult, ClipBlock, ClipContext, ClipFluid};

use super::vehicle_placement::{
    commit_placement_item, finish_placement_inventory, placement_cooldown,
    publish_placement_cooldown,
};

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

    fn normal_use_with_hand(&self, item: &Item, player: &Player, yaw: f32, pitch: f32, hand: Hand) {
        let stack = player.inventory().get_stack_in_hand(hand);
        if stack.item.id == item.id {
            self.place_from_hand(&stack, player, yaw, pitch, hand);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BoatItem {
    /// Vanilla: `BoatItem.use()` - raycasts to find placement position
    pub(crate) fn place_from_hand(
        &self,
        expected: &ItemStack,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: Hand,
    ) -> bool {
        self.place_from_hand_in_world(expected, player, yaw, pitch, hand, &player.world())
    }

    pub(crate) fn place_from_hand_in_world(
        &self,
        expected: &ItemStack,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: Hand,
        world: &Arc<World>,
    ) -> bool {
        const HIT_POSITION_TOLERANCE: f64 = 1.0e-7;

        if !Arc::ptr_eq(world, &player.world()) {
            return false;
        }
        let Some(slot_index) = player
            .inventory
            .equipment_slot_index(&crate::entity::hand_equipment_slot(hand))
        else {
            return false;
        };
        let stack = player.inventory.get_slot(slot_index);
        if stack.is_empty() || stack.uid != expected.uid || !stack.are_equal(expected) {
            return false;
        }
        let item = stack.item;
        let (group, duration) = placement_cooldown(&stack);
        if player.is_on_cooldown(group) {
            return false;
        }
        let Some(hit) = self.placement_hit(player, world, yaw, pitch) else {
            return false;
        };
        let hit_vec = hit.location;

        if Self::placement_blocked_at_eye(player, world, yaw, pitch) {
            return false;
        }

        let entity = Entity::new(world.clone(), hit_vec, Self::item_to_entity(item));
        let boat_entity: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(entity));
        crate::item::items::spawn_egg::apply_entity_variant(
            &stack,
            boat_entity.as_ref(),
            Some(player),
        );
        let boat = boat_entity.get_entity();
        boat.set_rotation(yaw, boat.pitch.load());
        if !world.no_vehicle_collision(boat_entity.as_ref(), boat.bounding_box.load()) {
            return false;
        }

        let Ok((consumed, gamemode, cooldown)) = world.spawn_placed_vehicle(&boat_entity, || {
            let gamemode = player.gamemode.load();
            if gamemode == GameMode::Spectator
                || !Arc::ptr_eq(world, &player.world())
                || player
                    .inventory
                    .equipment_slot_index(&crate::entity::hand_equipment_slot(hand))
                    != Some(slot_index)
            {
                return None;
            }
            // The spawn event saw this position. A changed ray target needs a new use.
            let live_hit = self.placement_hit(player, world, yaw, pitch)?;
            if live_hit.block_pos != hit.block_pos
                || live_hit.location.squared_distance_to_vec(&hit_vec)
                    > HIT_POSITION_TOLERANCE * HIT_POSITION_TOLERANCE
                || Self::placement_blocked_at_eye(player, world, yaw, pitch)
                || !world.no_vehicle_collision(boat_entity.as_ref(), boat.bounding_box.load())
            {
                return None;
            }
            let (consumed, cooldown) =
                commit_placement_item(player, slot_index, &stack, gamemode, group, duration)?;
            Some((consumed, gamemode, cooldown))
        }) else {
            return false;
        };
        world.emit_game_event_with_context(
            GameEvent::EntityPlace.name(),
            hit_vec,
            &crate::world::game_event::GameEventContext {
                source_entity: Some(player),
            },
        );
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
        finish_placement_inventory(player, slot_index, &stack, &consumed, gamemode);
        if let Some(cooldown) = cooldown {
            publish_placement_cooldown(player, group, cooldown);
        }
        true
    }

    fn placement_hit(
        &self,
        player: &Player,
        world: &World,
        yaw: f32,
        pitch: f32,
    ) -> Option<BlockHitResult> {
        let (start_pos, end_pos) = self.get_start_and_end_pos_with_rotation(player, yaw, pitch);
        // Vanilla BoatItem.use selects outlines and all fluids.
        let hit = world.clip(&ClipContext::new(
            start_pos,
            end_pos,
            ClipBlock::Outline,
            ClipFluid::Any,
        ));
        (!hit.is_miss).then_some(hit)
    }

    fn placement_blocked_at_eye(player: &Player, world: &World, yaw: f32, pitch: f32) -> bool {
        let direction = Vector3::rotation_vector(f64::from(pitch), f64::from(yaw));
        let search_box = player
            .get_entity()
            .bounding_box
            .load()
            .stretch(direction.multiply(5.0, 5.0, 5.0))
            .expand_all(1.0);
        let eye = player.eye_position();

        world.get_all_at_box(&search_box).iter().any(|entity| {
            let base = entity.get_entity();
            if base.entity_id == player.entity_id()
                || base.is_removed()
                || entity.is_spectator()
                || !entity.can_hit()
            {
                return false;
            }
            // Projectile.getPickRadius returns 1 for a pickable projectile.
            let pick_radius = if entity.get_projectile_owner().is_some() {
                1.0
            } else {
                0.0
            };
            base.bounding_box
                .load()
                .expand_all(pick_radius)
                .contains(eye)
        })
    }
}
