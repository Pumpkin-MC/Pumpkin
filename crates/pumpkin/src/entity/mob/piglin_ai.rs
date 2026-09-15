use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::Entity;
use crate::entity::living::LivingEntity;
use crate::entity::mob::piglin::PiglinEntity;

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
    pub fn is_admiring_disabled(piglin: &PiglinEntity) -> bool {
        piglin.is_admiring_disabled()
    }

    #[must_use]
    pub fn can_admire(piglin: &PiglinEntity, item_stack: &ItemStack) -> bool {
        if item_stack.is_empty() || Self::is_admiring_disabled(piglin) {
            return false;
        }
        if Self::is_barter_currency(item_stack) {
            return piglin.is_adult() && !piglin.is_admiring();
        }
        if piglin.is_admiring() {
            return false;
        }
        if Self::is_food(item_stack) {
            return !piglin.has_eaten_recently();
        }
        Self::is_loved_item(item_stack)
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
        if piglin.is_admiring_disabled() {
            return false;
        }
        if Self::is_barter_currency(item_stack) {
            return !piglin.is_admiring();
        }
        if Self::is_food(item_stack) {
            return !piglin.has_eaten_recently();
        }
        Self::is_loved_item(item_stack)
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

    pub fn throw_items(
        piglin: &PiglinEntity,
        items: Vec<ItemStack>,
        target_pos: Option<Vector3<f64>>,
    ) {
        let entity = &piglin.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let spawn_pos = Vector3::new(pos.x, pos.y + 1.0, pos.z);

        for item in items {
            if !item.is_empty() {
                let item_entity = crate::entity::item::ItemEntity::new(
                    Entity::new(world.clone(), spawn_pos, &EntityType::ITEM),
                    item,
                );
                if let Some(target) = target_pos {
                    let vel = Vector3::new(target.x - pos.x, target.y - pos.y, target.z - pos.z)
                        .normalize();
                    item_entity.get_entity().velocity.store(Vector3::new(
                        vel.x * 0.3,
                        0.3,
                        vel.z * 0.3,
                    ));
                }
                world.spawn_entity(Arc::new(item_entity));
            }
        }
    }
}
