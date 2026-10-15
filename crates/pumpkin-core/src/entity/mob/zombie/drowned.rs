use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::ranged_attack::RangedAttackGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity, RangedAttackMob};
use crate::entity::projectile::arrow::ArrowPickup;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;

pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);

        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged_arc: Arc<dyn RangedAttackMob> = mob_arc.clone();
            Arc::downgrade(&ranged_arc)
        };

        {
            let mut goal_selector = mob_arc
                .entity
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            goal_selector.add_goal(
                2,
                Box::new(DrownedTridentAttackGoal::new(ranged_weak, 1.0, 40, 10.0)),
            );

            let mut target_selector = mob_arc
                .entity
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::AXOLOTL,
                    true,
                ),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let entity = ZombieEntityBase::with_can_break_doors(entity, can_break_doors);
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);

        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged_arc: Arc<dyn RangedAttackMob> = mob_arc.clone();
            Arc::downgrade(&ranged_arc)
        };

        {
            let mut goal_selector = mob_arc
                .entity
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            goal_selector.add_goal(
                2,
                Box::new(DrownedTridentAttackGoal::new(ranged_weak, 1.0, 40, 10.0)),
            );

            let mut target_selector = mob_arc
                .entity
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::AXOLOTL,
                    true,
                ),
            );
        };

        mob_arc
    }

    pub fn perform_ranged_trident_attack(&self, target: &Arc<dyn EntityBase>) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load_full();

        let trident_stack = {
            let equipment = mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let item_in_hand = equipment.get(&EquipmentSlot::MAIN_HAND);
            if item_in_hand.item == &Item::TRIDENT {
                item_in_hand.clone()
            } else {
                ItemStack::new(1, &Item::TRIDENT)
            }
        };

        let target_entity = target.get_entity();
        let drowned_pos = entity.pos.load();
        let target_pos = target_entity.pos.load();

        let xd = target_pos.x - drowned_pos.x;
        let yd = (target_pos.y + f64::from(target_entity.height()) * 0.333_333_33)
            - (drowned_pos.y + f64::from(entity.height()) * 0.85);
        let zd = target_pos.z - drowned_pos.z;
        let dist_2d = (xd * xd + zd * zd).sqrt();

        let trident_base_entity = Entity::new(world.clone(), drowned_pos, &EntityType::TRIDENT);
        let trident = TridentEntity::new_shot(
            trident_base_entity,
            entity,
            trident_stack.clone(),
            ArrowPickup::Disallowed,
        );

        let difficulty = world.level_info.load().difficulty;
        let uncertainty = match difficulty {
            pumpkin_util::Difficulty::Hard => 2.0,
            pumpkin_util::Difficulty::Normal => 6.0,
            _ => 10.0,
        };

        trident.set_velocity(xd, yd + dist_2d * 0.2, zd, 1.6, uncertainty);
        trident.apply_on_projectile_spawned(&trident_stack);

        world.spawn_entity(Arc::new(trident));

        world.play_sound(
            Sound::EntityDrownedShoot,
            SoundCategory::Hostile,
            &drowned_pos,
        );
    }
}

pub struct DrownedTridentAttackGoal {
    inner: RangedAttackGoal,
}

impl DrownedTridentAttackGoal {
    #[must_use]
    pub const fn new(
        mob: Weak<dyn RangedAttackMob>,
        speed: f64,
        interval: i32,
        radius: f32,
    ) -> Self {
        Self {
            inner: RangedAttackGoal::new(mob, speed, interval, radius),
        }
    }

    fn has_trident(mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;
        let equipment = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment.get(&EquipmentSlot::MAIN_HAND).item == &Item::TRIDENT
    }
}

impl Goal for DrownedTridentAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::has_trident(mob) && self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        Self::has_trident(mob) && self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().set_attacking(true);
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().set_attacking(false);
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }
}

impl CustomSound for DrownedEntity {
    fn death_sound(&self) -> Option<Sound> {
        if self
            .entity
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed)
        {
            Some(Sound::EntityDrownedDeathWater)
        } else {
            Some(Sound::EntityDrownedDeath)
        }
    }

    fn hurt_sound(&self) -> Option<Sound> {
        if self
            .entity
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed)
        {
            Some(Sound::EntityDrownedHurtWater)
        } else {
            Some(Sound::EntityDrownedHurt)
        }
    }
}

impl RangedAttackMob for DrownedEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        self.perform_ranged_trident_attack(target);
    }
}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn spawn_as_baby(&self) -> bool {
        self.entity.spawn_as_baby()
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.mob_write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.mob_read_nbt(nbt);
    }

    fn get_base_experience_reward(&self) -> u32 {
        Mob::get_base_experience_reward(self.entity.as_ref())
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_valid_target(&self, target: Option<Arc<dyn EntityBase>>) -> Option<Arc<dyn EntityBase>> {
        let target = self.entity.as_valid_target(target)?;
        if self.entity.mob_entity.is_in_daylight()
            && !target.get_entity().touching_water.load(Ordering::Relaxed)
        {
            return None;
        }
        Some(target)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;

        if !entity.is_alive() {
            return;
        }

        // Vanilla okTarget check: in daylight, drop target if target is not in water
        if mob_entity.is_in_daylight()
            && let Some(target) = mob_entity.get_target()
            && !target.get_entity().touching_water.load(Ordering::Relaxed)
        {
            *mob_entity
                .target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }

        // Vanilla water swimming buoyancy: swim upwards towards target if submerged
        if entity.touching_water.load(Ordering::Relaxed)
            && let Some(target) = mob_entity.get_target()
        {
            let target_pos = target.get_entity().pos.load();
            let drowned_pos = entity.pos.load();
            if target_pos.y > drowned_pos.y {
                let mut vel = entity.velocity.load();
                vel.y += 0.04;
                if vel.y > 0.2 {
                    vel.y = 0.2;
                }
                entity.set_velocity(vel);
            }
        }
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        _difficulty: &RegionalDifficulty,
    ) {
        let living = &self.entity.mob_entity.living_entity;
        let mut equipment = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Vanilla: 10% chance to hold weapon (62.5% trident, 37.5% fishing rod)
        if rand::random::<f32>() < 0.10 {
            let rand_val = rand::random_range(0..16);
            let item = if rand_val < 10 {
                &Item::TRIDENT
            } else {
                &Item::FISHING_ROD
            };
            equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, item));
        }

        // Vanilla: 3% chance to hold nautilus shell in off-hand
        if rand::random::<f32>() < 0.03 {
            equipment.put(
                &EquipmentSlot::OFF_HAND,
                ItemStack::new(1, &Item::NAUTILUS_SHELL),
            );
        }
    }

    fn can_replace_current_item(
        &self,
        new_item: &ItemStack,
        current_item: &ItemStack,
        slot: &EquipmentSlot,
    ) -> bool {
        if current_item.item == &Item::NAUTILUS_SHELL {
            return false;
        }
        self.entity
            .can_replace_current_item(new_item, current_item, slot)
    }
}
