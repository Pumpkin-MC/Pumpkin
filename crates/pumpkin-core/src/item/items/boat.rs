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
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
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

    /// Gets entity dimensions for the boat type
    const fn get_entity_dimensions(entity_type: &EntityType) -> EntityDimensions {
        EntityDimensions::new(
            entity_type.dimension[0],
            entity_type.dimension[1],
            entity_type.eye_height,
        )
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

        let hit_vec = hit.location;

        // Vanilla: Check for entities in the path that would block placement
        // Get player's rotation vector stretched by 5.0 and expanded by 1.0
        let rotation_vec =
            Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)).multiply(5.0, 5.0, 5.0);
        let player_entity = player.get_entity().bounding_box.load();
        let search_box = player_entity.stretch(rotation_vec).expand_all(1.0);

        let entities = world.get_entities_at_box(&search_box);
        let player_eye_pos = player.eye_position();

        for entity in &entities {
            if entity.get_entity().entity_id == player.entity_id() {
                continue;
            }
            // Vanilla: EntityPredicates.CAN_HIT = !isSpectator() && canHit()
            if entity.is_spectator() || !entity.can_hit() {
                continue;
            }

            // Vanilla: expand by getTargetingMargin() (0.0 for most entities)
            let entity_box = entity.get_entity().bounding_box.load();
            // Check if entity's bounding box contains the player's eye position
            if entity_box.intersects(&BoundingBox::new(player_eye_pos, player_eye_pos)) {
                // Entity is blocking the line of sight at eye position
                return;
            }
        }

        // Create the boat entity
        let entity_type = Self::item_to_entity(item);
        let dimensions = Self::get_entity_dimensions(entity_type);
        let boat_box = BoundingBox::new_from_pos(hit_vec.x, hit_vec.y, hit_vec.z, &dimensions);

        // Vanilla: if (!world.isSpaceEmpty(lv7, lv7.getBoundingBox())) return FAIL
        if !world.is_space_empty(boat_box) {
            return;
        }

        // Check no collidable entities or other players occupy the space
        if world.get_entities_at_box(&boat_box).iter().any(|entity| {
            let ent = entity.get_entity();
            ent.entity_id != player.entity_id()
                && ent.entity_type != &EntityType::ITEM
                && ent.entity_type != &EntityType::EXPERIENCE_ORB
                && (entity.is_pushable() || entity.can_hit())
        }) || world
            .get_players_at_box(&boat_box)
            .iter()
            .any(|p| p.entity_id() != player.entity_id() && !p.is_spectator())
        {
            return;
        }

        // Create and spawn the boat
        let entity = Entity::new(world.clone(), hit_vec, entity_type);

        // Set yaw to player's yaw
        entity.set_rotation(yaw, 0.0);

        let boat_entity = Arc::new(BoatEntity::new(entity));
        world.spawn_entity(boat_entity);
        world.emit_game_event(GameEvent::EntityPlace.name(), hit_vec);

        let mut stack = player.inventory.get_stack_in_hand(hand);
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
