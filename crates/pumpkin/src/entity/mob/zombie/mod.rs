use super::{Mob, MobEntity};
use crate::entity::ai::goal::break_door::BreakDoorGoal;
use crate::entity::ai::goal::destroy_egg::DestroyEggGoal;
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::swim::SwimGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::{
    Entity,
    ai::goal::{Goal, active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use crate::world::World;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

pub mod drowned;
pub mod husk;
#[allow(clippy::module_inception)]
pub mod zombie;
pub mod zombie_villager;

pub struct ZombieEntityBase {
    pub mob_entity: MobEntity,
    pub can_break_doors: AtomicBool,
    /// `Zombie.inWaterTime`, `-1` when dry.
    pub in_water_time: AtomicI32,
    /// `Zombie.conversionTime`, `-1` when not converting.
    pub drowned_conversion_time: AtomicI32,
}

impl ZombieEntityBase {
    pub const IN_WATER_CONVERSION_DELAY: i32 = 600;
    pub const UNDER_WATER_CONVERSION_TIME: i32 = 300;

    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_can_break_doors(entity, false)
    }

    /// Shared zombie AI, optionally with door breaking.
    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let zombie = Self {
            mob_entity,
            can_break_doors: AtomicBool::new(can_break_doors),
            in_water_time: AtomicI32::new(-1),
            drowned_conversion_time: AtomicI32::new(-1),
        };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            if can_break_doors {
                goal_selector.add_goal(1, Box::new(BreakDoorGoal::default()));
            }
            goal_selector.add_goal(2, ZombieAttackGoal::new(1.0, false));
            goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
            target_selector.add_goal(
                5,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, true),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn can_break_doors(&self) -> bool {
        self.can_break_doors.load(Ordering::Relaxed)
    }

    /// `Zombie.isUnderWaterConverting`.
    #[must_use]
    pub fn is_under_water_converting(&self) -> bool {
        self.drowned_conversion_time.load(Ordering::Relaxed) >= 0
    }

    /// `Zombie.startUnderWaterConversion`.
    pub fn start_under_water_conversion(&self, ticks: i32) {
        self.drowned_conversion_time
            .store(ticks.max(0), Ordering::Relaxed);
        self.mob_entity.living_entity.entity.set_synced_data(
            pumpkin_data::tracked_data::zombie::DATA_DROWNED_CONVERSION_ID,
            true,
        );
    }

    /// `Zombie.tick` water conversion. Becomes `target` after the timers expire.
    pub fn tick_water_conversion(&self, target: &'static EntityType, sound_event: WorldEvent) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() || self.mob_entity.is_no_ai() {
            return;
        }

        if self.is_under_water_converting() {
            let remaining = self.drowned_conversion_time.fetch_sub(1, Ordering::Relaxed) - 1;
            if remaining < 0 {
                self.convert_to_zombie_type(target);
                if !entity.is_silent() {
                    entity
                        .world
                        .load()
                        .sync_world_event(sound_event, entity.block_pos.load(), 0);
                }
            }
        } else if entity.is_submerged_in_water() {
            let time = self.in_water_time.fetch_add(1, Ordering::Relaxed) + 1;
            if time >= Self::IN_WATER_CONVERSION_DELAY {
                self.start_under_water_conversion(Self::UNDER_WATER_CONVERSION_TIME);
            }
        } else {
            self.in_water_time.store(-1, Ordering::Relaxed);
        }
    }

    /// `Zombie.convertToZombieType`.
    fn convert_to_zombie_type(&self, target: &'static EntityType) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();

        let converted = crate::entity::r#type::from_type(
            target,
            entity.pos.load(),
            &world,
            uuid::Uuid::new_v4(),
        );
        self.mob_entity.copy_conversion_state(converted.as_ref());
        if self.can_break_doors()
            && let Some(converted_mob) = converted.get_mob()
        {
            let mut nbt = NbtCompound::new();
            nbt.put_bool("CanBreakDoors", true);
            converted_mob.mob_read_nbt(&nbt);
        }

        world.spawn_entity(converted.clone());
        self.mob_entity
            .copy_conversion_equipment(converted.as_ref());
        entity.remove();
    }

    pub fn set_can_break_doors(&self, can_break_doors: bool, mob: &dyn Mob) {
        if self
            .can_break_doors
            .swap(can_break_doors, Ordering::Relaxed)
            != can_break_doors
        {
            let mut stopped = {
                let mut goal_selector = self
                    .mob_entity
                    .goals_selector
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if can_break_doors {
                    goal_selector.add_goal(1, Box::new(BreakDoorGoal::default()));
                    Vec::new()
                } else {
                    goal_selector.remove_goals::<BreakDoorGoal>()
                }
            };
            for goal in &mut stopped {
                goal.stop(mob);
            }
        }
    }
}

impl Mob for ZombieEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        // Default armor slots (super.populateDefaultEquipmentSlots)
        if rand::random::<f32>()
            < MobEntity::MAX_WEARING_ARMOR_CHANCE * difficulty.special_multiplier
        {
            let mut armor_type = rand::random_range(0..3);
            for _ in 1..=3 {
                if rand::random::<f32>() < MobEntity::WEARING_ARMOR_UPGRADE_MATERIAL_CHANCE {
                    armor_type += 1;
                }
            }

            let partial_chance = if difficulty.base_difficulty == Difficulty::Hard {
                0.1f32
            } else {
                0.25f32
            };

            let living = &self.mob_entity.living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut first = true;

            for slot in &MobEntity::EQUIPMENT_POPULATION_ORDER {
                let current = equipment.get(slot);
                if !first && rand::random::<f32>() < partial_chance {
                    break;
                }
                first = false;
                if current.is_empty()
                    && let Some(item) = MobEntity::get_equipment_for_slot(slot, armor_type)
                {
                    equipment.put(slot, ItemStack::new(1, item));
                }
            }
        }

        let weapon_chance = if difficulty.base_difficulty == Difficulty::Hard {
            0.05f32
        } else {
            0.01f32
        };
        if rand::random::<f32>() < weapon_chance {
            let r = rand::random_range(0..6);
            let weapon_item = match r {
                0 => &Item::IRON_SWORD,
                1 => &Item::IRON_SPEAR,
                _ => &Item::IRON_SHOVEL,
            };
            let living = &self.mob_entity.living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, weapon_item));
        }
    }

    /// Writes `CanBreakDoors`, `InWaterTime`, `DrownedConversionTime`.
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if self.can_break_doors() {
            nbt.put_bool("CanBreakDoors", true);
        }
        let in_water_time = if self.mob_entity.living_entity.entity.is_in_water() {
            self.in_water_time.load(Ordering::Relaxed)
        } else {
            -1
        };
        nbt.put_int("InWaterTime", in_water_time);
        nbt.put_int(
            "DrownedConversionTime",
            self.drowned_conversion_time.load(Ordering::Relaxed),
        );
    }

    /// Reads `CanBreakDoors`, `InWaterTime`, `DrownedConversionTime`.
    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(can_break_doors) = nbt.get_bool("CanBreakDoors") {
            self.set_can_break_doors(can_break_doors, self);
        }
        if let Some(in_water_time) = nbt.get_int("InWaterTime") {
            self.in_water_time.store(in_water_time, Ordering::Relaxed);
        }
        if let Some(conversion_time) = nbt.get_int("DrownedConversionTime")
            && conversion_time > -1
        {
            self.start_under_water_conversion(conversion_time);
        }
    }
}
