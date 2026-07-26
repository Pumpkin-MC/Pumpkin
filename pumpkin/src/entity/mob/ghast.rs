use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        Controls, Goal, GoalFuture, active_target::ActiveTargetGoal,
        fireball_attack::FireballAttackGoal, random_float::RandomFloatGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct GhastEntity {
    pub mob_entity: MobEntity,
    pub is_charging: AtomicBool,
    pub explosion_power: AtomicU8,
}

impl GhastEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ghast = Self {
            mob_entity,
            is_charging: AtomicBool::new(false),
            explosion_power: AtomicU8::new(1),
        };

        let mob_arc = Arc::new(ghast);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 Ghast.registerGoals
            goal_selector.add_goal(5, RandomFloatGoal::new());
            goal_selector.add_goal(7, Box::new(GhastLookGoal::new(mob_weak)));
            goal_selector.add_goal(7, FireballAttackGoal::new());

            // Vanilla also filters |target.y - ghast.y| <= 4.0 (Y-band TODO).
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn set_charging(&self, charging: bool) {
        self.is_charging.store(charging, Ordering::Relaxed);
    }

    pub fn is_charging(&self) -> bool {
        self.is_charging.load(Ordering::Relaxed)
    }
}

impl NBTStorage for GhastEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for GhastEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}

pub struct GhastLookGoal {
    goal_control: Controls,
    mob_weak: Weak<dyn Mob>,
}

impl GhastLookGoal {
    #[must_use]
    pub fn new(mob_weak: Weak<dyn Mob>) -> Self {
        Self {
            goal_control: Controls::LOOK,
            mob_weak,
        }
    }
}

impl Goal for GhastLookGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            let target_opt = mob_entity.target.lock().await.clone();

            if let Some(target) = target_opt {
                let mob_pos = mob_entity.living_entity.entity.pos.load();
                let target_pos = target.get_entity().pos.load();

                if mob_pos.squared_distance_to_vec(&target_pos) < 4096.0 {
                    let mut look_control = mob_entity.look_control.lock().unwrap();
                    look_control.look_at(mob, target_pos.x, target_pos.y, target_pos.z);
                }
            } else {
                let velocity = mob_entity.living_entity.entity.velocity.load();
                if velocity.x != 0.0 || velocity.z != 0.0 {
                    let yaw = (-f64::atan2(velocity.x, velocity.z) * (180.0 / std::f64::consts::PI))
                        as f32;
                    mob_entity.living_entity.entity.yaw.store(yaw);
                }
            }
            let _ = &self.mob_weak;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
