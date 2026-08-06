use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::dimension::Dimension;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    r#type::from_type,
};

pub struct PiglinEntity {
    pub mob_entity: MobEntity,
    pub time_in_overworld: AtomicI32,
}

impl PiglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            time_in_overworld: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Piglins use crossbows or swords, but for now we give them melee
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::WITHER_SKELETON,
                    true,
                ),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WITHER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for PiglinEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int(
                "TimeInOverworld",
                self.time_in_overworld.load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.time_in_overworld.store(
                nbt.get_int("TimeInOverworld").unwrap_or(0),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for PiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = caller.get_entity();
            let overworld_time = self.time_in_overworld.load(Ordering::Relaxed);
            let world = entity.world.load_full();

            if world.dimension == Dimension::THE_NETHER {
                if overworld_time > 0 {
                    self.time_in_overworld
                        .store(overworld_time - 1, Ordering::Relaxed);
                }
                return;
            }

            let new_time = overworld_time + 1;
            self.time_in_overworld.store(new_time, Ordering::Relaxed);

            if new_time >= 300 {
                convert_to_zombified(caller, &EntityType::ZOMBIFIED_PIGLIN).await;
            }
        })
    }
}

pub(super) async fn convert_to_zombified(
    caller: &Arc<dyn EntityBase>,
    target_type: &'static EntityType,
) {
    let entity = caller.get_entity();
    let pos = entity.pos.load();
    let world = entity.world.load_full();
    let uuid = entity.entity_uuid;
    let yaw = entity.yaw.load();
    let pitch = entity.pitch.load();
    let age = entity.age.load(Ordering::Relaxed);
    let custom_name = entity.custom_name.load().clone();
    let custom_name_visible = entity.custom_name_visible.load(Ordering::Relaxed);
    let velocity = entity.velocity.load();

    let living = caller
        .get_living_entity()
        .expect("Piglin must be a living entity");
    let health = living.health.load();
    let equipment = {
        let eq = living.entity_equipment.lock().await;
        eq.clone()
    };
    let active_effects = {
        let effects = living.active_effects.lock().await;
        effects.clone()
    };

    entity.remove().await;

    let zombified = from_type(target_type, pos, &world, uuid);
    let zombified_entity = zombified.get_entity();
    let zombified_living = zombified
        .get_living_entity()
        .expect("Zombified Piglin must be a living entity");

    zombified_living.health.store(health);
    zombified_entity.age.store(age, Ordering::Relaxed);
    zombified_entity.custom_name.store(custom_name);
    zombified_entity
        .custom_name_visible
        .store(custom_name_visible, Ordering::Relaxed);
    zombified_entity.yaw.store(yaw);
    zombified_entity.pitch.store(pitch);
    zombified_entity.velocity.store(velocity);

    {
        let mut eq = zombified_living.entity_equipment.lock().await;
        *eq = equipment;
        drop(eq);
    };
    {
        let mut effects = zombified_living.active_effects.lock().await;
        *effects = active_effects;
    };
    zombified_living
        .add_effect(Effect {
            effect_type: &StatusEffect::NAUSEA,
            duration: 200,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        })
        .await;

    world.spawn_entity(zombified).await;
}
