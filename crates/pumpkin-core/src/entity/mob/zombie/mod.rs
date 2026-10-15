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
    Entity, EntityBase,
    ai::goal::{Goal, active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

pub mod drowned;
pub mod husk;
#[allow(clippy::module_inception)]
pub mod zombie;
pub mod zombie_villager;

pub struct ZombieEntityBase {
    pub mob_entity: MobEntity,
    pub can_break_doors: AtomicBool,
    pub is_baby: AtomicBool,
    pub reinforcement_chance: AtomicCell<f32>,
}

impl ZombieEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_can_break_doors(entity, false)
    }

    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mut rng = rand::rng();
        let reinforcement_chance = if rng.random::<f32>() < 0.05 {
            0.55
        } else {
            rng.random_range(0.0..0.1)
        };
        let zombie = Self {
            mob_entity,
            can_break_doors: AtomicBool::new(can_break_doors),
            is_baby: AtomicBool::new(false),
            reinforcement_chance: AtomicCell::new(reinforcement_chance),
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

            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true).alerting_others_except(|entity_type| {
                        entity_type == &EntityType::ZOMBIFIED_PIGLIN
                    }),
                ),
            );
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

impl ZombieEntityBase {
    /// Vanilla `Zombie.setBaby`: the baby state is only the synced flag, it never ages up.
    /// Vanilla baby zombies receive the +50% `SPEED_MODIFIER_BABY` and 2.5x XP
    /// (`Zombie.getBaseExperienceReward`).
    pub fn set_baby(&self, baby: bool) {
        self.mob_entity.set_baby_flag(
            &self.is_baby,
            pumpkin_data::tracked_data::zombie::BABY,
            baby,
        );
        let living = &self.mob_entity.living_entity;
        if baby {
            let modifier = crate::entity::attributes::Modifier {
                id: "minecraft:baby".to_string(),
                amount: 0.5,
                operation: crate::entity::attributes::ModifierOperation::MultiplyBase,
            };
            living.update_attribute(&pumpkin_data::attributes::Attributes::MOVEMENT_SPEED, |inst| {
                inst.add_or_replace_modifier(modifier);
            });
        } else {
            living.update_attribute(&pumpkin_data::attributes::Attributes::MOVEMENT_SPEED, |inst| {
                inst.remove_modifier("minecraft:baby");
            });
        }
        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![pumpkin_data::attributes::Attributes::MOVEMENT_SPEED],
        );
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.is_baby.load(Ordering::Relaxed)
    }
}

impl Mob for ZombieEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn spawn_as_baby(&self) -> bool {
        self.set_baby(true);
        true
    }

    fn get_base_experience_reward(&self) -> u32 {
        let base = self
            .mob_entity
            .living_entity
            .entity
            .entity_type
            .experience_reward;
        if self.is_baby() {
            (base as f32 * 2.5) as u32
        } else {
            base
        }
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

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        // Vanilla Zombie.addAdditionalSaveData; shared by every zombie-family mob.
        nbt.put_bool("IsBaby", self.is_baby());
        if self.can_break_doors() {
            nbt.put_bool("CanBreakDoors", true);
        }
        nbt.put_float("SpawnReinforcementsChance", self.reinforcement_chance.load());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_baby(nbt.get_bool("IsBaby").unwrap_or(false));
        if let Some(can_break_doors) = nbt.get_bool("CanBreakDoors") {
            self.set_can_break_doors(can_break_doors, self);
        }
        if let Some(chance) = nbt.get_float("SpawnReinforcementsChance") {
            self.reinforcement_chance.store(chance);
        }
    }
}

impl ZombieEntityBase {
    pub fn try_spawn_reinforcements(&self, source: Option<&dyn EntityBase>) {
        let world = self.mob_entity.living_entity.entity.world.load_full();
        let level_info = world.level_info.load();
        if level_info.difficulty != Difficulty::Hard {
            return;
        }

        let target = self.mob_entity.get_target().or_else(|| {
            source.and_then(|s| {
                let id = s.get_entity().entity_id;
                world
                    .get_player_by_id(id)
                    .map(|p| p as Arc<dyn EntityBase>)
                    .or_else(|| world.get_entity_by_id(id))
            })
        });
        let Some(target) = target else {
            return;
        };

        let mut rng = rand::rng();
        let chance = self.reinforcement_chance.load();
        if rng.random::<f32>() >= chance {
            return;
        }

        let zombie_pos = self.mob_entity.living_entity.entity.pos.load();
        let entity_type = &self.mob_entity.living_entity.entity.entity_type;
        for _ in 0..50 {
            let offset_x = (rng.random_range(7..=40) * if rng.random_bool(0.5) { 1 } else { -1 }) as f64;
            let offset_y = rng.random_range(-7..=7) as f64;
            let offset_z = (rng.random_range(7..=40) * if rng.random_bool(0.5) { 1 } else { -1 }) as f64;
            let spawn_pos = Vector3::new(
                zombie_pos.x + offset_x,
                zombie_pos.y + offset_y,
                zombie_pos.z + offset_z,
            );
            let block_pos = BlockPos(Vector3::new(
                spawn_pos.x.floor() as i32,
                spawn_pos.y.floor() as i32,
                spawn_pos.z.floor() as i32,
            ));
            let state = world.get_block_state(&block_pos);
            let up = world.get_block_state(&block_pos.up());
            let down = world.get_block_state(&block_pos.down());
            if !state.is_air() || !up.is_air() || down.is_air() {
                continue;
            }
            if world.get_nearest_player(spawn_pos, 7.0, |_| true).is_some() {
                continue;
            }

            let reinforcement = crate::entity::r#type::from_type(
                entity_type,
                spawn_pos,
                &world,
                Uuid::new_v4(),
            );
            crate::entity::mob::spawn::finalize_spawn(&reinforcement, &world, None);
            if let Some(mob) = reinforcement.get_mob() {
                mob.set_mob_target(Some(target.clone()));
            }
            world.spawn_entity(reinforcement);
            self.reinforcement_chance.store((chance - 0.05).max(0.0));
            break;
        }
    }
}
