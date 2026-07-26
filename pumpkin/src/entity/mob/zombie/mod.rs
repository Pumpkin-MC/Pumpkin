use super::{Mob, MobEntity};
use crate::entity::ai::goal::destroy_egg::DestroyEggGoal;
use crate::entity::ai::goal::door_interact::BreakDoorGoal;
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::swim::SwimGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::{
    Entity, EntityBase, NBTStorage, NbtFuture,
    ai::goal::{active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Weak};

pub mod drowned;
pub mod husk;
#[allow(clippy::module_inception)]
pub mod zombie;
pub mod zombie_villager;

pub struct ZombieEntityBase {
    pub mob_entity: MobEntity,
    /// Vanilla `Zombie.DATA_BABY_ID`. Rolled at construction (5%), overridden
    /// by the saved `IsBaby` tag on load.
    pub is_baby: AtomicBool,
}

impl ZombieEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        // Vanilla Zombie.randomizeReinforcementsChance: base = random * 0.1.
        mob_entity.living_entity.set_attribute_base(
            &Attributes::SPAWN_REINFORCEMENTS,
            rand::random::<f64>() * 0.1,
        );
        // Vanilla getSpawnAsBabyOdds: 5% of zombie-family spawns are babies.
        let is_baby = rand::random::<f32>() < 0.05;
        if is_baby {
            Self::apply_baby_speed(&mob_entity, true);
        }
        let zombie = Self {
            mob_entity,
            is_baby: AtomicBool::new(is_baby),
        };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 Zombie.registerGoals + addBehaviourGoals
            // (SpearUseGoal / MoveThroughVillage TODO)
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Vanilla adds/removes breakDoorGoal at priority 1 with setCanBreakDoors;
            // the goal itself checks the flag so registration can stay static.
            goal_selector.add_goal(
                1,
                Box::new(BreakDoorGoal::new(|difficulty| {
                    difficulty == Difficulty::Hard
                })),
            );
            // ZombieAttackGoal priority 3 (vanilla addBehaviourGoals)
            goal_selector.add_goal(3, ZombieAttackGoal::new(1.0, false));
            // ZombieAttackTurtleEggGoal priority 4
            goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3));
            // WaterAvoidingRandomStrollGoal priority 7
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            // HurtByTargetGoal.setAlertOthers(ZombifiedPiglin) — handled on ZombifiedPiglin via JoinAnger(ZOMBIE*)
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            // AbstractVillager: checkVisibility=false in vanilla
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
            // Turtle baby-on-land selector TODO — still target turtles
            target_selector.add_goal(
                5,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, true),
            );
        };

        mob_arc
    }

    /// Vanilla `SPEED_MODIFIER_BABY`: babies move 50% faster.
    fn apply_baby_speed(mob_entity: &MobEntity, baby: bool) {
        let living = &mob_entity.living_entity;
        let default_speed = living
            .entity
            .entity_type
            .attributes
            .iter()
            .find(|attribute| attribute.0.id == Attributes::MOVEMENT_SPEED.id)
            .map_or(0.23, |attribute| attribute.1);
        living.set_attribute_base(
            &Attributes::MOVEMENT_SPEED,
            if baby {
                default_speed * 1.5
            } else {
                default_speed
            },
        );
    }

    pub fn set_baby(&self, baby: bool) {
        use std::sync::atomic::Ordering;
        if self.is_baby.swap(baby, Ordering::Relaxed) != baby {
            Self::apply_baby_speed(&self.mob_entity, baby);
        }
    }
}

impl NBTStorage for ZombieEntityBase {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_bool("CanBreakDoors", self.mob_entity.can_break_doors());
            nbt.put_bool(
                "IsBaby",
                self.is_baby.load(std::sync::atomic::Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if let Some(can_break_doors) = nbt.get_bool("CanBreakDoors") {
                self.mob_entity
                    .set_can_break_doors_from_nbt(can_break_doors);
            }
            // Saved state wins over the construction-time baby roll.
            self.set_baby(nbt.get_bool("IsBaby").unwrap_or(false));
        })
    }
}

impl Mob for ZombieEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn is_mob_baby(&self) -> bool {
        self.is_baby.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn supports_break_door_goal(&self) -> bool {
        true
    }

    /// Vanilla `Zombie.hurtServer` reinforcement call: on Hard difficulty a hurt
    /// zombie may summon another of its kind nearby, at a shrinking chance.
    fn on_damage<'a>(
        &'a self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let living = &self.mob_entity.living_entity;
            let entity = &living.entity;
            let world = entity.world.load_full();

            {
                let level_info = world.level_info.load();
                if level_info.difficulty != Difficulty::Hard
                    || !level_info.game_rules.spawn_monsters
                {
                    return;
                }
            }

            let chance = living.get_attribute_value(&Attributes::SPAWN_REINFORCEMENTS);
            if f64::from(rand::random::<f32>()) >= chance {
                return;
            }

            // Vanilla uses the current target, falling back to the attacker.
            let target: Option<Arc<dyn EntityBase>> = {
                let current = self.mob_entity.target.lock().await.clone();
                match current {
                    Some(target) => Some(target),
                    None => source
                        .filter(|s| s.get_living_entity().is_some())
                        .and_then(|s| world.get_entity_by_id(s.get_entity().entity_id)),
                }
            };
            let Some(target) = target else {
                return;
            };

            let origin = entity.block_pos.load();
            let entity_type = entity.entity_type;
            for _ in 0..50 {
                // Scope the (non-Send) thread rng away from the awaits below.
                let (dx, dy, dz) = {
                    let mut rng = rand::rng();
                    let mut offset = || rng.random_range(7i32..=40) * rng.random_range(-1i32..=1);
                    (offset(), offset(), offset())
                };
                let spawn_pos = BlockPos::new(origin.0.x + dx, origin.0.y + dy, origin.0.z + dz);

                // Standable: solid floor, two passable blocks.
                let floor = world.get_block_state(&spawn_pos.down());
                let feet = world.get_block_state(&spawn_pos);
                let head = world.get_block_state(&spawn_pos.up());
                if !floor.is_side_solid(pumpkin_data::BlockDirection::Up)
                    || feet.is_solid()
                    || head.is_solid()
                {
                    continue;
                }

                let spawn_center = spawn_pos.to_f64().add_raw(0.5, 0.0, 0.5);
                // Vanilla rejects positions with a player within 7 blocks.
                if world.get_closest_player(spawn_center, 7.0).is_some() {
                    continue;
                }

                let reinforcement: Arc<dyn Mob> = match entity_type.id {
                    id if id == EntityType::HUSK.id => {
                        husk::HuskEntity::new(Entity::new(world.clone(), spawn_center, entity_type))
                    }
                    id if id == EntityType::DROWNED.id => drowned::DrownedEntity::new(Entity::new(
                        world.clone(),
                        spawn_center,
                        entity_type,
                    )),
                    id if id == EntityType::ZOMBIE_VILLAGER.id => {
                        zombie_villager::ZombieVillagerEntity::new(Entity::new(
                            world.clone(),
                            spawn_center,
                            entity_type,
                        ))
                    }
                    _ => zombie::ZombieEntity::new(Entity::new(
                        world.clone(),
                        spawn_center,
                        &EntityType::ZOMBIE,
                    )),
                };
                reinforcement.set_mob_target(Some(target)).await;

                // Both caller and callee lose 5% future reinforcement chance.
                living.set_attribute_base(
                    &Attributes::SPAWN_REINFORCEMENTS,
                    (chance - 0.05).max(0.0),
                );
                let reinforcement_living = &reinforcement.get_mob_entity().living_entity;
                let callee_chance =
                    reinforcement_living.get_attribute_value(&Attributes::SPAWN_REINFORCEMENTS);
                reinforcement_living.set_attribute_base(
                    &Attributes::SPAWN_REINFORCEMENTS,
                    (callee_chance - 0.05).max(0.0),
                );

                let reinforcement_base: Arc<dyn EntityBase> = reinforcement;
                world.spawn_entity(reinforcement_base).await;
                break;
            }
        })
    }
}
