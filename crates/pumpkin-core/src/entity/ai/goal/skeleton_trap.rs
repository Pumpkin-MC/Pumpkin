use std::sync::{Arc, atomic::Ordering::Relaxed};

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::enchantment_provider::EnchantmentProvider;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::passive::abstract_horse::AbstractHorse;
use crate::entity::passive::skeleton_horse::SkeletonHorseEntity;
use crate::entity::r#type::from_type;
use crate::world::World;

const LIGHTNING_INVULNERABLE_TICKS: i32 = 60;
const TRIGGER_RANGE: f64 = 10.0;
const EXTRA_HORSES: usize = 3;
const PUSH_SPREAD: f64 = 1.1485;

pub struct SkeletonTrapGoal;

impl Goal for SkeletonTrapGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        entity
            .world
            .load()
            .get_nearest_player(entity.pos.load(), TRIGGER_RANGE, |player| {
                !player.is_spectator() && player.get_entity().is_alive()
            })
            .is_some()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(horse) = mob.cast_any().downcast_ref::<SkeletonHorseEntity>() else {
            return;
        };
        let entity = mob.get_entity();
        let world = entity.world.load_full();
        let pos = entity.pos.load();
        let difficulty = RegionalDifficulty::at(&world, pos);
        horse.set_trap(false);
        horse.set_tamed(true);
        horse.set_age(0);
        world.strike_lightning(pos, true);
        let Some(vehicle) = world.get_entity_by_id(entity.entity_id) else {
            return;
        };
        let skeleton = create_skeleton(&world, &difficulty, pos);
        if !world.spawn_entity(skeleton.clone()) {
            return;
        }
        entity.add_passenger(vehicle, skeleton);
        for _ in 0..EXTRA_HORSES {
            let other_horse = create_horse(&world, pos);
            let Some(other_mob) = other_horse.get_mob() else {
                continue;
            };
            let skeleton = create_skeleton(&world, &difficulty, pos);
            other_mob.get_mob_entity().add_pending_rider(skeleton);
            let mut rng = mob.get_random();
            let push = Vector3::new(
                triangle(&mut rng, PUSH_SPREAD),
                0.0,
                triangle(&mut rng, PUSH_SPREAD),
            );
            other_horse.get_entity().add_velocity(push);
            world.spawn_entity(other_horse);
        }
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

fn triangle(rng: &mut impl rand::Rng, spread: f64) -> f64 {
    spread * (rng.random::<f64>() - rng.random::<f64>())
}

fn create_horse(world: &Arc<World>, pos: Vector3<f64>) -> Arc<dyn EntityBase> {
    let horse = from_type(&EntityType::SKELETON_HORSE, pos, world, Uuid::new_v4());
    finalize_spawn(&horse, world, EntitySpawnReason::Triggered, None);
    if let Some(mob) = horse.get_mob() {
        make_persistent_and_invulnerable(mob);
    }
    if let Some(skeleton_horse) = horse.cast_any().downcast_ref::<SkeletonHorseEntity>() {
        skeleton_horse.set_tamed(true);
        skeleton_horse.set_age(0);
    }
    horse
}

fn make_persistent_and_invulnerable(mob: &dyn Mob) {
    let mob_entity = mob.get_mob_entity();
    mob_entity
        .living_entity
        .hurt_cooldown
        .store(LIGHTNING_INVULNERABLE_TICKS, Relaxed);
    mob_entity.persistence_required.store(true, Relaxed);
}

fn create_skeleton(
    world: &Arc<World>,
    difficulty: &RegionalDifficulty,
    pos: Vector3<f64>,
) -> Arc<dyn EntityBase> {
    let skeleton = from_type(&EntityType::SKELETON, pos, world, Uuid::new_v4());
    finalize_spawn(&skeleton, world, EntitySpawnReason::Triggered, None);
    if let Some(mob) = skeleton.get_mob() {
        make_persistent_and_invulnerable(mob);
        let mob_entity = mob.get_mob_entity();
        let head_empty = equipped(mob, &EquipmentSlot::HEAD).is_empty();
        if head_empty {
            mob_entity.set_item_slot(&EquipmentSlot::HEAD, ItemStack::new(1, &Item::IRON_HELMET));
        }
        enchant(mob, &EquipmentSlot::MAIN_HAND, difficulty);
        enchant(mob, &EquipmentSlot::HEAD, difficulty);
    }
    skeleton
}

fn enchant(skeleton: &dyn Mob, slot: &EquipmentSlot, difficulty: &RegionalDifficulty) {
    let mut stack = equipped(skeleton, slot);
    stack.remove_data_component(DataComponent::Enchantments);
    EnchantmentProvider::MOB_SPAWN_EQUIPMENT
        .apply_to_stack(&mut stack, difficulty.special_multiplier);
    skeleton.get_mob_entity().set_item_slot(slot, stack);
}

fn equipped(mob: &dyn Mob, slot: &EquipmentSlot) -> ItemStack {
    mob.get_mob_entity()
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(slot)
}
