use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::ai::random_pos::land_random_pos;
use crate::entity::item::ItemEntity;
use crate::entity::living::LivingEntity;
use crate::entity::mob::piglin::{PiglinActivity, PiglinEntity};
use crate::entity::player::Player;

pub struct PiglinAi;

impl PiglinAi {
    pub const REPELLENT_DETECTION_RANGE_HORIZONTAL: i32 = 8;
    pub const REPELLENT_DETECTION_RANGE_VERTICAL: i32 = 4;
    pub const BARTERING_ITEM: &'static Item = &Item::GOLD_INGOT;
    pub const PLAYER_ANGER_RANGE: f64 = 16.0;
    pub const ANGER_DURATION: i32 = 600;
    pub const ADMIRE_DURATION: i32 = 119;
    pub const ADMIRING_DISABLED_DURATION: i32 = 400;
    pub const EAT_COOLDOWN: i32 = 200;
    pub const BABY_FLEE_DURATION: i32 = 100;
    pub const CELEBRATION_TIME: i32 = 300;
    pub const MIN_TIME_BETWEEN_HUNTS: i32 = 600;
    pub const MAX_TIME_BETWEEN_HUNTS: i32 = 2400;
    pub const DESIRED_DISTANCE_FROM_ZOMBIFIED: f64 = 6.0;
    pub const PROBABILITY_OF_CELEBRATION_DANCE: f32 = 0.1;
    pub const ITEM_SCAN_RANGE: f64 = 32.0;
    pub const ITEM_SCAN_RANGE_Y: f64 = 16.0;
    pub const SENSOR_SCAN_INTERVAL: i64 = 20;
    pub const MAX_DISTANCE_TO_WALK_TO_ITEM: f64 = 9.0;
    pub const MAX_TIME_TRYING_TO_REACH_ITEM: i32 = 200;
    pub const DISABLE_WALK_TO_ADMIRE_DURATION: i32 = 200;
    pub const TARGETING_RANGE: f64 = 16.0;
    pub const MIN_VISIBILITY_DISTANCE: f64 = 2.0;
    pub const THROW_SPEED: f64 = 0.3;
    pub const THROW_HAND_Y_DISTANCE_FROM_EYE: f64 = 0.3;
    pub const RANDOM_THROW_POS_RANGE_HORIZONTAL: i32 = 4;
    pub const RANDOM_THROW_POS_RANGE_VERTICAL: i32 = 2;

    #[must_use]
    pub const fn is_barter_currency(item_stack: &ItemStack) -> bool {
        item_stack.item.id == Self::BARTERING_ITEM.id
    }

    #[must_use]
    pub fn is_loved_item(item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_LOVED)
    }

    #[must_use]
    pub fn is_food(item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_FOOD)
    }

    #[must_use]
    pub const fn is_zombified(entity_type: &EntityType) -> bool {
        entity_type.id == EntityType::ZOMBIFIED_PIGLIN.id || entity_type.id == EntityType::ZOGLIN.id
    }

    #[must_use]
    pub fn wants_to_dance(killed_target_type: &EntityType) -> bool {
        if killed_target_type.id != EntityType::HOGLIN.id {
            return false;
        }
        rand::random::<f32>() < Self::PROBABILITY_OF_CELEBRATION_DANCE
    }

    #[must_use]
    pub fn is_wearing_safe_armor(entity: &LivingEntity) -> bool {
        let Ok(guard) = entity.entity_equipment.try_lock() else {
            return false;
        };

        for slot in [
            EquipmentSlot::HEAD,
            EquipmentSlot::CHEST,
            EquipmentSlot::LEGS,
            EquipmentSlot::FEET,
        ] {
            if let Some(stack) = guard.equipment.get(&slot)
                && !stack.is_empty()
                && (stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_SAFE_ARMOR)
                    || stack.item.has_tag(&tag::Item::MINECRAFT_PIGLIN_LOVED))
            {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn is_holding_loved_item(entity: &LivingEntity) -> bool {
        let Ok(guard) = entity.entity_equipment.try_lock() else {
            return false;
        };
        for slot in [EquipmentSlot::MAIN_HAND, EquipmentSlot::OFF_HAND] {
            if let Some(stack) = guard.equipment.get(&slot)
                && !stack.is_empty()
                && Self::is_loved_item(stack)
            {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn is_player_holding_loved_item(player: &Player) -> bool {
        let inventory = player.inventory();
        Self::is_loved_item(&inventory.held_item())
            || Self::is_loved_item(&inventory.off_hand_item())
    }

    #[must_use]
    pub fn is_admiring_disabled(piglin: &PiglinEntity) -> bool {
        piglin.is_admiring_disabled()
    }

    #[must_use]
    pub fn can_admire(piglin: &PiglinEntity, item_stack: &ItemStack) -> bool {
        !Self::is_admiring_disabled(piglin)
            && !piglin.is_admiring()
            && piglin.is_adult()
            && Self::is_barter_currency(item_stack)
    }

    #[must_use]
    pub fn is_not_holding_loved_item_in_off_hand(piglin: &PiglinEntity) -> bool {
        let off_hand = piglin.off_hand_item();
        off_hand.is_empty() || !Self::is_loved_item(&off_hand)
    }

    #[must_use]
    pub fn wants_to_pickup(piglin: &PiglinEntity, item_stack: &ItemStack) -> bool {
        if piglin.is_baby()
            && item_stack
                .item
                .has_tag(&tag::Item::MINECRAFT_IGNORED_BY_PIGLIN_BABIES)
        {
            return false;
        }
        if item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_PIGLIN_REPELLENTS)
        {
            return false;
        }
        if piglin.is_admiring_disabled() && piglin.has_attack_target() {
            return false;
        }
        if Self::is_barter_currency(item_stack) {
            return Self::is_not_holding_loved_item_in_off_hand(piglin);
        }

        let has_space = piglin.can_add_to_inventory(item_stack);
        if item_stack.item.id == Item::GOLD_NUGGET.id {
            return has_space;
        }
        if Self::is_food(item_stack) {
            return !piglin.has_eaten_recently() && has_space;
        }
        if !Self::is_loved_item(item_stack) {
            return piglin.can_replace_current_item_for(item_stack);
        }
        Self::is_not_holding_loved_item_in_off_hand(piglin) && has_space
    }

    #[must_use]
    pub fn get_barter_response_items() -> Vec<ItemStack> {
        let roll = rand::random_range(0..459);
        match roll {
            0..5 => vec![ItemStack::new(1, &Item::ENCHANTED_BOOK)],
            5..13 => vec![ItemStack::new(1, &Item::IRON_BOOTS)],
            13..21 => vec![ItemStack::new(1, &Item::SPLASH_POTION)],
            21..39 => vec![ItemStack::new(1, &Item::POTION)],
            39..49 => vec![ItemStack::new(
                rand::random_range(10..=36),
                &Item::IRON_NUGGET,
            )],
            49..59 => vec![ItemStack::new(
                rand::random_range(2..=4),
                &Item::ENDER_PEARL,
            )],
            59..79 => vec![ItemStack::new(rand::random_range(3..=9), &Item::STRING)],
            79..99 => vec![ItemStack::new(rand::random_range(5..=12), &Item::QUARTZ)],
            99..139 => vec![ItemStack::new(1, &Item::OBSIDIAN)],
            139..179 => vec![ItemStack::new(
                rand::random_range(1..=3),
                &Item::CRYING_OBSIDIAN,
            )],
            179..219 => vec![ItemStack::new(1, &Item::FIRE_CHARGE)],
            219..259 => vec![ItemStack::new(rand::random_range(2..=4), &Item::LEATHER)],
            259..299 => vec![ItemStack::new(rand::random_range(2..=8), &Item::SOUL_SAND)],
            299..339 => vec![ItemStack::new(
                rand::random_range(2..=8),
                &Item::NETHER_BRICK,
            )],
            339..379 => vec![ItemStack::new(
                rand::random_range(6..=12),
                &Item::SPECTRAL_ARROW,
            )],
            379..419 => vec![ItemStack::new(rand::random_range(8..=16), &Item::GRAVEL)],
            _ => vec![ItemStack::new(
                rand::random_range(8..=16),
                &Item::BLACKSTONE,
            )],
        }
    }

    #[must_use]
    pub fn get_sound_for_activity(piglin: &PiglinEntity, activity: PiglinActivity) -> Sound {
        if activity == PiglinActivity::Fight {
            return Sound::EntityPiglinAngry;
        }
        let world = piglin.mob_entity.living_entity.entity.world.load();
        if piglin.is_converting(&world) {
            return Sound::EntityPiglinRetreat;
        }
        match activity {
            PiglinActivity::AdmireItem => Sound::EntityPiglinAdmiringItem,
            PiglinActivity::Celebrate => Sound::EntityPiglinCelebrate,
            _ if Self::sees_player_holding_loved_item(piglin) => Sound::EntityPiglinJealous,
            _ if piglin.is_near_repellent() => Sound::EntityPiglinRetreat,
            _ => Sound::EntityPiglinAmbient,
        }
    }

    fn sees_player_holding_loved_item(piglin: &PiglinEntity) -> bool {
        piglin
            .nearest_visible_player()
            .is_some_and(|player| Self::is_player_holding_loved_item(&player))
    }

    /// Whether the piglin can see and would notice `target`, invisibility and sneaking included.
    #[must_use]
    pub fn is_entity_targetable(piglin: &PiglinEntity, target: &Player) -> bool {
        let entity = &piglin.mob_entity.living_entity.entity;
        let target_entity = target.get_entity();
        if !target.living_entity.is_part_of_game() {
            return false;
        }

        let is_attack_target = piglin
            .mob_entity
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|current| current.get_entity().entity_id == target_entity.entity_id);
        let modifier = if is_attack_target {
            1.0
        } else {
            Self::visibility_percent(target)
        };
        let visibility_distance =
            (Self::TARGETING_RANGE * modifier).max(Self::MIN_VISIBILITY_DISTANCE);
        let distance_sq = entity
            .pos
            .load()
            .squared_distance_to_vec(&target_entity.pos.load());
        if distance_sq > visibility_distance * visibility_distance {
            return false;
        }

        entity
            .world
            .load()
            .raycast(
                entity.get_eye_pos(),
                target_entity.get_eye_pos(),
                |block_pos, w| w.get_block_state(block_pos).is_solid(),
            )
            .is_none()
    }

    /// How visible `target` is to a piglin, from 1.0 down to 0.1.
    fn visibility_percent(target: &Player) -> f64 {
        let target_entity = target.get_entity();
        let mut percent = 1.0;
        if target_entity.is_sneaking() {
            percent *= 0.8;
        }
        let head = {
            let equipment = target
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if target_entity
                .invisible
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let worn = [
                    EquipmentSlot::HEAD,
                    EquipmentSlot::CHEST,
                    EquipmentSlot::LEGS,
                    EquipmentSlot::FEET,
                ]
                .iter()
                .filter(|slot| !equipment.get(slot).is_empty())
                .count();
                let cover = (worn as f64 / 4.0).max(0.1);
                percent *= 0.7 * cover;
            }
            equipment.get(&EquipmentSlot::HEAD)
        };
        if head.item.id == Item::PIGLIN_HEAD.id {
            percent *= 0.5;
        }
        percent
    }

    pub fn throw_items(piglin: &PiglinEntity, items: Vec<ItemStack>) {
        match piglin.nearest_visible_player() {
            Some(player) => {
                Self::throw_items_toward_pos(piglin, items, player.get_entity().pos.load());
            }
            None => Self::throw_items_toward_random_pos(piglin, items),
        }
    }

    pub fn throw_items_toward_random_pos(piglin: &PiglinEntity, items: Vec<ItemStack>) {
        Self::throw_items_toward_pos(piglin, items, Self::random_nearby_pos(piglin));
    }

    fn throw_items_toward_pos(piglin: &PiglinEntity, items: Vec<ItemStack>, target: Vector3<f64>) {
        if items.is_empty() {
            return;
        }
        let living = &piglin.mob_entity.living_entity;
        living.swing_off_hand();

        let entity = &living.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let hand_pos = Vector3::new(
            pos.x,
            entity.get_eye_y() - Self::THROW_HAND_Y_DISTANCE_FROM_EYE,
            pos.z,
        );
        let direction =
            Vector3::new(target.x - pos.x, target.y + 1.0 - pos.y, target.z - pos.z).normalize();
        let velocity = Vector3::new(
            direction.x * Self::THROW_SPEED,
            direction.y * Self::THROW_SPEED,
            direction.z * Self::THROW_SPEED,
        );

        for item in items {
            if item.is_empty() {
                continue;
            }
            let item_entity = ItemEntity::new_with_velocity(
                Entity::new(world.clone(), hand_pos, &EntityType::ITEM),
                item,
                velocity,
                ItemEntity::DEFAULT_PICKUP_DELAY,
            );
            world.spawn_entity(Arc::new(item_entity));
        }
    }

    fn random_nearby_pos(piglin: &PiglinEntity) -> Vector3<f64> {
        land_random_pos(
            piglin,
            Self::RANDOM_THROW_POS_RANGE_HORIZONTAL,
            Self::RANDOM_THROW_POS_RANGE_VERTICAL,
        )
        .unwrap_or_else(|| piglin.mob_entity.living_entity.entity.pos.load())
    }
}
