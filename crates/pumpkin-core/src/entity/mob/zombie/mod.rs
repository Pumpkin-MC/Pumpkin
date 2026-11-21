use super::conversion::{ConversionParams, ConversionTracker, convert_to};
use super::equipment::RegionalDifficulty;
use super::spawn::{EntitySpawnReason, RANDOM_SPAWN_BONUS_ID, SpawnGroupData, finalize_spawn};
use super::{Mob, MobEntity, MobMetadata};
use crate::entity::ai::goal::break_door::BreakDoorGoal;
use crate::entity::ai::goal::destroy_egg::DestroyEggGoal;
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;
use crate::entity::ai::goal::move_through_village::MoveThroughVillageGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::spear_use::SpearUseGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::mob::zombie::zombie_villager::ZombieVillagerEntity;
use crate::entity::passive::chicken::ChickenEntity;
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::r#type::{check_spawn_rules, from_type};
use crate::entity::{
    Entity, EntityBase,
    ai::behavior::hunting::{self, Hunt},
    ai::goal::look_at_entity::LookAtEntityGoal,
    ai::target_match::TargetMatch,
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::World;
use crate::world::natural_spawner::is_spawn_position_ok;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

pub mod drowned;
pub mod husk;
#[allow(clippy::module_inception)]
pub mod zombie;
pub mod zombie_villager;

const ZOMBIE_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true),
    Hunt::new(3, TargetMatch::VILLAGER_LIKE, false),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), true),
    hunting::baby_turtles(5, true),
];

const SPEED_MODIFIER_BABY_ID: &str = "minecraft:baby";
const SPEED_MODIFIER_BABY: f64 = 0.5;
const REINFORCEMENT_CALLER_CHARGE_ID: &str = "minecraft:reinforcement_caller_charge";
const REINFORCEMENT_CALLEE_CHARGE_ID: &str = "minecraft:reinforcement_callee_charge";
const REINFORCEMENT_CHARGE: f64 = 0.05;
const LEADER_ZOMBIE_BONUS_ID: &str = "minecraft:leader_zombie_bonus";
const ZOMBIE_RANDOM_SPAWN_BONUS_ID: &str = "minecraft:zombie_random_spawn_bonus";
pub const ZOMBIE_LEADER_CHANCE: f32 = 0.05;
const REINFORCEMENT_ATTEMPTS: i32 = 50;
const REINFORCEMENT_RANGE_MAX: i32 = 40;
const REINFORCEMENT_RANGE_MIN: i32 = 7;
const REINFORCEMENT_PLAYER_DISTANCE: f64 = 7.0;
const BREAK_DOOR_CHANCE: f32 = 0.1;
const BABY_CHANCE: f32 = 0.05;
const CHICKEN_JOCKEY_CHANCE: f32 = 0.05;
const HALLOWEEN_HEAD_CHANCE: f32 = 0.25;
const JACK_O_LANTERN_CHANCE: f32 = 0.1;
const DROWNING_AFFLICTION_TICKS: i32 = 600;
const DROWNING_CONVERSION_TICKS: i32 = 300;

/// Shared state and behaviour of every zombie-family mob.
pub struct ZombieEntityBase {
    pub mob_entity: MobEntity,
    pub can_break_doors: AtomicBool,
    pub is_baby: AtomicBool,
    drowning: ConversionTracker,
}

impl ZombieEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_behaviour_goals(entity, Self::add_behaviour_goals)
    }

    /// `HurtByTargetGoal` that alerts every nearby zombie but zombified piglins.
    #[must_use]
    pub fn revenge_goal() -> RevengeGoal {
        RevengeGoal::new(true)
            .alerting_others_except(|entity_type| entity_type == &EntityType::ZOMBIFIED_PIGLIN)
    }

    /// Movement, attack and target goals; the drowned and zombified piglin replace them.
    pub fn add_behaviour_goals(mob_entity: &MobEntity) {
        let goal_selector = &mob_entity.goals_selector;
        goal_selector.add_goal(2, Box::new(SpearUseGoal::new(1.0, 1.0, 10.0, 2.0)));
        goal_selector.add_goal(3, ZombieAttackGoal::new(1.0, false));
        goal_selector.add_goal(
            6,
            Box::new(MoveThroughVillageGoal::new(1.0, true, 4, |mob| {
                mob.as_zombie().is_some_and(Self::can_break_doors)
            })),
        );
        goal_selector.add_goal(7, Box::new(WanderAroundGoal::water_avoiding(1.0)));
        let target_selector = &mob_entity.target_selector;
        target_selector.add_goal(1, Box::new(Self::revenge_goal()));
        hunting::apply(target_selector, mob_entity, ZOMBIE_HUNTS);
    }

    pub fn with_behaviour_goals(entity: Entity, add_behaviour_goals: fn(&MobEntity)) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let zombie = Self {
            mob_entity,
            can_break_doors: AtomicBool::new(false),
            is_baby: AtomicBool::new(false),
            drowning: ConversionTracker::new(
                tracked_data::zombie::DATA_DROWNED_CONVERSION_ID,
                "InWaterTime",
                DROWNING_AFFLICTION_TICKS,
                "DrownedConversionTime",
                DROWNING_CONVERSION_TICKS,
            ),
        };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let goal_selector = &mob_arc.mob_entity.goals_selector;
        goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3));
        goal_selector.add_goal(
            8,
            LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
        );
        goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        add_behaviour_goals(&mob_arc.mob_entity);

        mob_arc
    }

    #[must_use]
    pub fn can_break_doors(&self) -> bool {
        self.can_break_doors.load(Ordering::Relaxed)
    }

    pub fn set_can_break_doors(&self, can_break_doors: bool, mob: &dyn Mob) {
        let mut navigator = self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if navigator.can_navigate_ground() {
            if self
                .can_break_doors
                .swap(can_break_doors, Ordering::Relaxed)
                != can_break_doors
            {
                navigator.set_can_open_doors(can_break_doors);
                drop(navigator);
                if can_break_doors {
                    self.mob_entity
                        .goals_selector
                        .add_goal(1, Box::new(BreakDoorGoal::default()));
                } else {
                    self.mob_entity
                        .goals_selector
                        .remove_goal::<BreakDoorGoal>(mob);
                }
            }
        } else if self.can_break_doors.swap(false, Ordering::Relaxed) {
            drop(navigator);
            self.mob_entity
                .goals_selector
                .remove_goal::<BreakDoorGoal>(mob);
        }
    }

    /// The baby state is only the synced flag; zombies never age up.
    pub fn set_baby(&self, baby: bool) {
        self.mob_entity
            .set_baby_flag(&self.is_baby, tracked_data::zombie::BABY, baby);
        self.mob_entity
            .living_entity
            .update_attribute(&Attributes::MOVEMENT_SPEED, |speed| {
                speed.remove_modifier(SPEED_MODIFIER_BABY_ID);
                if baby {
                    speed.add_or_replace_modifier(Modifier {
                        id: SPEED_MODIFIER_BABY_ID.to_string(),
                        amount: SPEED_MODIFIER_BABY,
                        operation: ModifierOperation::MultiplyBase,
                    });
                }
            });
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.is_baby.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn is_under_water_converting(&self) -> bool {
        self.drowning.is_converting()
    }

    #[must_use]
    pub fn can_hold_item(&self, stack: &ItemStack) -> bool {
        !(stack.item.has_tag(&pumpkin_data::tag::Item::MINECRAFT_EGGS)
            && self.is_baby()
            && self.mob_entity.living_entity.entity.has_vehicle())
    }

    fn difficulty_here(&self, world: &Arc<World>) -> RegionalDifficulty {
        RegionalDifficulty::at(world, self.mob_entity.living_entity.entity.pos.load())
    }

    pub fn finalize_zombie_spawn(
        &self,
        mob: &dyn Mob,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut rng = mob.get_random();
        let group_data = mob.finalize_spawn_super(group_data);
        let difficulty = self.difficulty_here(world);
        let difficulty_modifier = difficulty.special_multiplier;
        if reason != EntitySpawnReason::Conversion {
            self.mob_entity.set_can_pick_up_loot(
                rng.random::<f32>() < MobEntity::MAX_PICKUP_LOOT_CHANCE * difficulty_modifier,
            );
        }

        let group_data = group_data.or_else(|| {
            Some(SpawnGroupData::Zombie {
                is_baby: rng.random::<f32>() < BABY_CHANCE,
                can_spawn_jockey: true,
            })
        });
        if let Some(SpawnGroupData::Zombie {
            is_baby,
            can_spawn_jockey,
        }) = group_data
        {
            if is_baby {
                self.set_baby(true);
                if can_spawn_jockey {
                    self.roll_chicken_jockey(mob, world);
                }
            }
            self.set_can_break_doors(
                rng.random::<f32>() < difficulty_modifier * BREAK_DOOR_CHANCE,
                mob,
            );
            if reason != EntitySpawnReason::Conversion {
                mob.populate_default_equipment_slots(world, &difficulty);
                mob.populate_default_equipment_enchantments(&difficulty);
            }
        }

        if self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::HEAD)
            .is_empty()
            && is_halloween()
            && rng.random::<f32>() < HALLOWEEN_HEAD_CHANCE
        {
            let head = if rng.random::<f32>() < JACK_O_LANTERN_CHANCE {
                &Item::JACK_O_LANTERN
            } else {
                &Item::CARVED_PUMPKIN
            };
            self.mob_entity
                .set_item_slot(&EquipmentSlot::HEAD, ItemStack::new(1, head));
            self.mob_entity
                .living_entity
                .equipment_drop_chances
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(EquipmentSlot::HEAD, 0.0);
        }

        self.handle_attributes(mob, difficulty_modifier, reason);
        group_data
    }

    fn roll_chicken_jockey(&self, mob: &dyn Mob, world: &Arc<World>) {
        let mut rng = mob.get_random();
        let entity = &self.mob_entity.living_entity.entity;
        let ride_nearby_chicken = rng.random::<f32>() < CHICKEN_JOCKEY_CHANCE;
        if ride_nearby_chicken {
            let area = entity.bounding_box.load().expand(5.0, 3.0, 5.0);
            let chicken = world
                .get_entities_of_type_at_box(&area, |entity_type| {
                    entity_type == &EntityType::CHICKEN
                })
                .into_iter()
                .find(|chicken| {
                    let chicken = chicken.get_entity();
                    chicken.is_alive() && !chicken.has_passengers() && !chicken.has_vehicle()
                });
            if let Some(chicken) = chicken {
                set_chicken_jockey(chicken.as_ref());
                self.mob_entity.set_pending_vehicle(chicken);
            }
        } else if rng.random::<f32>() < CHICKEN_JOCKEY_CHANCE {
            let chicken = from_type(
                &EntityType::CHICKEN,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            );
            chicken.get_entity().set_rotation(entity.yaw.load(), 0.0);
            finalize_spawn(&chicken, world, EntitySpawnReason::Jockey, None);
            set_chicken_jockey(chicken.as_ref());
            self.mob_entity.set_pending_vehicle(chicken);
        }
    }

    /// Spawn bonuses: reinforcement chance, knockback resistance, follow range and the leader roll.
    pub fn handle_attributes(
        &self,
        mob: &dyn Mob,
        difficulty_modifier: f32,
        reason: EntitySpawnReason,
    ) {
        let mut rng = mob.get_random();
        let living = &self.mob_entity.living_entity;
        let reinforcements_chance = if mob.get_entity().entity_type == &EntityType::ZOMBIFIED_PIGLIN
        {
            0.0
        } else {
            rng.random::<f64>() * f64::from(0.1f32)
        };
        living.set_attribute_base(&Attributes::SPAWN_REINFORCEMENTS, reinforcements_chance);
        let knockback_bonus = rng.random::<f64>() * f64::from(0.05f32);
        living.update_attribute(&Attributes::KNOCKBACK_RESISTANCE, |attribute| {
            attribute.add_or_replace_permanent_modifier(Modifier {
                id: RANDOM_SPAWN_BONUS_ID.to_string(),
                amount: knockback_bonus,
                operation: ModifierOperation::Add,
            });
        });
        let follow_range_modifier = rng.random::<f64>() * 1.5 * f64::from(difficulty_modifier);
        if follow_range_modifier > 1.0 {
            living.update_attribute(&Attributes::FOLLOW_RANGE, |attribute| {
                attribute.add_or_replace_permanent_modifier(Modifier {
                    id: ZOMBIE_RANDOM_SPAWN_BONUS_ID.to_string(),
                    amount: follow_range_modifier,
                    operation: ModifierOperation::MultiplyTotal,
                });
            });
        }

        if rng.random::<f32>() < difficulty_modifier * ZOMBIE_LEADER_CHANCE {
            let reinforcement_bonus = rng.random::<f64>() * 0.25 + 0.5;
            living.update_attribute(&Attributes::SPAWN_REINFORCEMENTS, |attribute| {
                attribute.add_or_replace_permanent_modifier(Modifier {
                    id: LEADER_ZOMBIE_BONUS_ID.to_string(),
                    amount: reinforcement_bonus,
                    operation: ModifierOperation::Add,
                });
            });
            let health_bonus = rng.random::<f64>() * 3.0 + 1.0;
            living.update_attribute(&Attributes::MAX_HEALTH, |attribute| {
                attribute.add_or_replace_permanent_modifier(Modifier {
                    id: LEADER_ZOMBIE_BONUS_ID.to_string(),
                    amount: health_bonus,
                    operation: ModifierOperation::MultiplyTotal,
                });
            });
            if !matches!(
                reason,
                EntitySpawnReason::Conversion
                    | EntitySpawnReason::Load
                    | EntitySpawnReason::DimensionTravel
            ) {
                living.set_health(living.get_max_health());
            }
            self.set_can_break_doors(true, mob);
        }
    }

    /// Converts to `converts_to` after enough time with its eyes under water.
    pub fn tick_drowning(
        &self,
        mob: &dyn Mob,
        converts_to: &'static EntityType,
        conversion_sound: WorldEvent,
    ) {
        let afflicted = self.mob_entity.living_entity.entity.is_submerged_in_water();
        self.drowning.tick(mob, afflicted, || {
            let world = mob.get_entity().world.load_full();
            convert_to(
                mob,
                converts_to,
                ConversionParams::single(true, true),
                |converted| {
                    if let Some(zombie) = converted.as_zombie() {
                        let difficulty_modifier = zombie.difficulty_here(&world).special_multiplier;
                        zombie.handle_attributes(
                            converted,
                            difficulty_modifier,
                            EntitySpawnReason::Conversion,
                        );
                    }
                    let converted = converted.get_entity();
                    if !converted.is_silent() {
                        world.sync_world_event(conversion_sound, converted.block_pos.load(), 0);
                    }
                },
            );
        });
    }

    /// Hard difficulty can call reinforcements from up to 40 blocks away when hurt.
    #[must_use]
    pub fn has_global_tick_reach(&self) -> bool {
        let world = self.mob_entity.living_entity.entity.world.load();
        world.level_info.load().difficulty == Difficulty::Hard
            && self
                .mob_entity
                .living_entity
                .get_attribute_value(&Attributes::SPAWN_REINFORCEMENTS)
                > 0.0
    }

    pub fn summon_reinforcements(&self, mob: &dyn Mob, attacker: Option<&dyn EntityBase>) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        if world.level_info.load().difficulty != Difficulty::Hard {
            return;
        }
        let Some(target) = self.mob_entity.get_target().or_else(|| {
            let attacker = attacker?;
            attacker.get_living_entity()?;
            let id = attacker.get_entity().entity_id;
            world.get_entity_by_id(id)
        }) else {
            return;
        };
        let mut rng = mob.get_random();
        if f64::from(rng.random::<f32>())
            >= self
                .mob_entity
                .living_entity
                .get_attribute_value(&Attributes::SPAWN_REINFORCEMENTS)
            || !world.should_spawn_monsters()
        {
            return;
        }

        let origin = entity.block_pos.load();
        let entity_type = entity.entity_type;
        let reinforcement = from_type(entity_type, entity.pos.load(), &world, Uuid::new_v4());
        let Some(reinforcement_mob) = reinforcement.get_mob() else {
            return;
        };
        let can_spawn_in_liquids = entity_type == &EntityType::DROWNED;
        let mut offset = || {
            rng.random_range(REINFORCEMENT_RANGE_MIN..=REINFORCEMENT_RANGE_MAX)
                * rng.random_range(-1..=1)
        };
        for _ in 0..REINFORCEMENT_ATTEMPTS {
            let spawn_pos = BlockPos::new(
                origin.0.x + offset(),
                origin.0.y + offset(),
                origin.0.z + offset(),
            );
            if !is_spawn_position_ok(&world, &spawn_pos, entity_type)
                || !check_spawn_rules(
                    entity_type,
                    &world,
                    &spawn_pos,
                    world.is_thundering(),
                    EntitySpawnReason::Reinforcement,
                )
            {
                continue;
            }
            let spawn_vec = spawn_pos.0.to_f64();
            let reinforcement_entity = reinforcement.get_entity();
            reinforcement_entity.set_pos(spawn_vec);
            let bounds = reinforcement_entity.bounding_box.load();
            if world.has_nearby_alive_player(spawn_vec, REINFORCEMENT_PLAYER_DISTANCE)
                || !world.is_unobstructed(reinforcement_entity)
                || !world.no_collision(reinforcement.as_ref(), bounds)
                || (!can_spawn_in_liquids && world.contains_any_liquid(bounds))
            {
                continue;
            }
            reinforcement_mob.set_mob_target(Some(target), TargetReason::Unknown);
            finalize_spawn(
                &reinforcement,
                &world,
                EntitySpawnReason::Reinforcement,
                None,
            );
            world.spawn_entity(reinforcement.clone());
            self.mob_entity.living_entity.update_attribute(
                &Attributes::SPAWN_REINFORCEMENTS,
                |attribute| {
                    let existing = attribute
                        .modifiers
                        .iter()
                        .find(|modifier| modifier.id == REINFORCEMENT_CALLER_CHARGE_ID)
                        .map_or(0.0, |modifier| modifier.amount);
                    attribute.remove_modifier(REINFORCEMENT_CALLER_CHARGE_ID);
                    attribute.add_or_replace_permanent_modifier(Modifier {
                        id: REINFORCEMENT_CALLER_CHARGE_ID.to_string(),
                        amount: existing - REINFORCEMENT_CHARGE,
                        operation: ModifierOperation::Add,
                    });
                },
            );
            reinforcement_mob
                .get_mob_entity()
                .living_entity
                .update_attribute(&Attributes::SPAWN_REINFORCEMENTS, |attribute| {
                    attribute.add_or_replace_permanent_modifier(Modifier {
                        id: REINFORCEMENT_CALLEE_CHARGE_ID.to_string(),
                        amount: -f64::from(REINFORCEMENT_CHARGE as f32),
                        operation: ModifierOperation::Add,
                    });
                });
            break;
        }
    }

    /// Melee hit with an empty hand while burning can set the target on fire.
    pub fn do_hurt_target(&self, mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        // The mob itself, so the victim's death can credit it with the kill.
        let attacker: &dyn EntityBase = mob;
        if !self.mob_entity.try_attack(attacker, target) {
            return false;
        }
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        let difficulty = self.difficulty_here(&world).effective_difficulty;
        let main_hand_empty = self
            .mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .is_empty();
        if main_hand_empty
            && entity.is_on_fire()
            && mob.get_random().random::<f32>() < difficulty * 0.3
        {
            target.set_on_fire_for((2 * difficulty as i32) as f32);
        }
        true
    }

    /// A villager killed on Normal (half the time) or Hard becomes a zombie villager.
    pub fn killed_entity(&self, mob: &dyn Mob, killed: &dyn EntityBase) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        let difficulty = world.level_info.load().difficulty;
        if !matches!(difficulty, Difficulty::Normal | Difficulty::Hard) {
            return true;
        }
        let Some(villager) = killed.cast_any().downcast_ref::<VillagerEntity>() else {
            return true;
        };
        if difficulty != Difficulty::Hard && mob.get_random().random::<bool>() {
            return true;
        }
        convert_villager_to_zombie_villager(entity, &world, villager).is_none()
    }

    pub fn on_offspring_spawned_from_egg(&self, offspring: &dyn Mob) {
        let world = self.mob_entity.living_entity.entity.world.load_full();
        let difficulty_modifier =
            RegionalDifficulty::at(&world, offspring.get_entity().pos.load()).special_multiplier;
        offspring.get_mob_entity().set_can_pick_up_loot(
            rand::random::<f32>() < MobEntity::MAX_PICKUP_LOOT_CHANCE * difficulty_modifier,
        );
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("IsBaby", self.is_baby());
        nbt.put_bool("CanBreakDoors", self.can_break_doors());
        self.drowning.write_nbt(
            nbt,
            self.mob_entity.living_entity.entity.is_submerged_in_water(),
        );
    }

    pub fn read_nbt(&self, mob: &dyn Mob, nbt: &NbtCompound) {
        self.set_baby(nbt.get_bool("IsBaby").unwrap_or(false));
        self.set_can_break_doors(nbt.get_bool("CanBreakDoors").unwrap_or(false), mob);
        self.drowning
            .read_nbt(&self.mob_entity.living_entity.entity, nbt);
    }
}

fn set_chicken_jockey(chicken: &dyn EntityBase) {
    if let Some(chicken) = chicken.cast_any().downcast_ref::<ChickenEntity>() {
        chicken.set_chicken_jockey(true);
    }
}

fn convert_villager_to_zombie_villager(
    zombie: &Entity,
    world: &Arc<World>,
    villager: &VillagerEntity,
) -> Option<Arc<dyn EntityBase>> {
    convert_to(
        villager,
        &EntityType::ZOMBIE_VILLAGER,
        ConversionParams::single(true, true),
        |converted| {
            let zombie_villager = converted.cast_any().downcast_ref::<ZombieVillagerEntity>();
            if let Some(zombie_villager) = zombie_villager {
                zombie_villager.set_villager_data_finalized(villager.get_villager_data_finalized());
            }
            converted.finalize_spawn(
                world,
                EntitySpawnReason::Conversion,
                Some(SpawnGroupData::Zombie {
                    is_baby: false,
                    can_spawn_jockey: true,
                }),
            );
            if let Some(zombie_villager) = zombie_villager {
                zombie_villager.copy_villager(villager);
            }
            if !zombie.is_silent() {
                world.sync_world_event(WorldEvent::SoundZombieInfected, zombie.block_pos.load(), 0);
            }
        },
    )
}

/// The server's local date is October 31st.
fn is_halloween() -> bool {
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    now.month() == time::Month::October && now.day() == 31
}

pub fn wants_to_pick_up<M: Mob + ?Sized>(mob: &M, stack: &ItemStack) -> bool {
    stack.item.id != Item::GLOW_INK_SAC.id && mob.can_hold_item(stack)
}

impl Mob for ZombieEntityBase {
    fn xp_reward(&self) -> u32 {
        zombie_xp_reward(
            self.mob_entity
                .living_entity
                .entity
                .entity_type
                .experience_reward,
            self.is_baby(),
        )
    }

    fn mob_metadata(&self) -> Option<MobMetadata> {
        self.is_baby().then(|| MobMetadata::new().baby(true))
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_zombie(&self) -> Option<&ZombieEntityBase> {
        Some(self)
    }

    fn spawn_as_baby(&self) -> bool {
        self.set_baby(true);
        true
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
        self.write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_nbt(self, nbt);
    }
}

/// Baby zombies are worth two and a half times the experience.
#[must_use]
pub fn zombie_xp_reward(xp_reward: u32, is_baby: bool) -> u32 {
    if is_baby {
        (f64::from(xp_reward) * 2.5) as u32
    } else {
        xp_reward
    }
}

/// `Mob` hooks every zombie-family wrapper forwards to its [`ZombieEntityBase`].
#[macro_export]
macro_rules! zombie_mob_hooks {
    ($base:ident) => {
        fn get_mob_entity(&self) -> &$crate::entity::mob::MobEntity {
            &self.$base.mob_entity
        }

        fn as_zombie(&self) -> Option<&$crate::entity::mob::zombie::ZombieEntityBase> {
            Some(&self.$base)
        }

        fn xp_reward(&self) -> u32 {
            self.$base.xp_reward()
        }

        fn can_hold_item(&self, stack: &pumpkin_data::item_stack::ItemStack) -> bool {
            self.$base.can_hold_item(stack)
        }

        fn spawn_as_baby(&self) -> bool {
            self.$base.spawn_as_baby()
        }

        fn on_damage_with_context(
            &self,
            damage_type: pumpkin_data::damage::DamageType,
            source: Option<&dyn $crate::entity::EntityBase>,
            cause: Option<&dyn $crate::entity::EntityBase>,
        ) -> bool {
            self.on_damage(damage_type, source);
            self.$base.summon_reinforcements(self, cause.or(source));
            true
        }

        fn killed_entity(&self, killed: &dyn $crate::entity::EntityBase) -> bool {
            self.$base.killed_entity(self, killed)
        }

        fn has_global_tick_reach(&self) -> bool {
            self.$base.has_global_tick_reach()
        }

        fn on_offspring_spawned_from_egg(
            &self,
            _spawner: &$crate::entity::player::Player,
            offspring: &dyn $crate::entity::mob::Mob,
        ) {
            self.$base.on_offspring_spawned_from_egg(offspring);
        }
    };
}
