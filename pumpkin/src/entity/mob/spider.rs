use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        Controls, Goal, GoalFuture, active_target::ActiveTargetGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

struct SpiderAttackGoal {
    melee: MeleeAttackGoal,
}

impl SpiderAttackGoal {
    fn new() -> Self {
        Self {
            melee: MeleeAttackGoal::new(1.0, true),
        }
    }
}

impl Goal for SpiderAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.melee.can_start(mob)
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let world = entity.world.load();
            let brightness =
                world.get_sky_light_level(&entity.get_eye_pos().to_block_pos()) as f32 / 15.0;
            if brightness >= 0.5 && mob.get_random().random_range(0..100) == 0 {
                mob.set_mob_target(None).await;
                return false;
            }
            self.melee.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.melee.start(mob)
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.melee.stop(mob)
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.melee.tick(mob)
    }

    fn controls(&self) -> Controls {
        self.melee.controls()
    }
}

struct SpiderTargetGoal {
    inner: Box<ActiveTargetGoal>,
}

impl SpiderTargetGoal {
    fn new(mob: &MobEntity, target_type: &'static EntityType) -> Self {
        Self {
            inner: ActiveTargetGoal::with_default(mob, target_type, true),
        }
    }

    fn is_dark(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        world.get_sky_light_level(&entity.get_eye_pos().to_block_pos()) as f32 / 15.0 < 0.5
    }
}

impl Goal for SpiderTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !Self::is_dark(mob) {
                self.inner.set_target(None);
                return false;
            }
            self.inner.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::is_dark(mob) && self.inner.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.start(mob)
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.stop(mob)
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

struct SpiderLeapGoal;

impl Goal for SpiderLeapGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            if !entity.on_ground.load(std::sync::atomic::Ordering::Relaxed)
                || mob.get_random().random_range(0..5) != 0
            {
                return false;
            }
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return false;
            };
            let distance = entity
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());
            (4.0..=16.0).contains(&distance)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            !mob.get_mob_entity()
                .living_entity
                .entity
                .on_ground
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            let target_pos = target.get_entity().pos.load();
            let pos = entity.pos.load();
            let delta = Vector3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z);
            if delta.length_squared() <= 1.0e-7 {
                return;
            }
            let movement = delta.normalize() * 0.4 + entity.velocity.load() * 0.2;
            entity.set_velocity(Vector3::new(movement.x, 0.4, movement.z));
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}

pub struct SpiderEntity {
    pub mob_entity: MobEntity,
}

impl SpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let spider = Self { mob_entity };
        let mob_arc = Arc::new(spider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(SpiderLeapGoal));
            goal_selector.add_goal(3, Box::new(SpiderAttackGoal::new()));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                Box::new(SpiderTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                )),
            );
            target_selector.add_goal(
                3,
                Box::new(SpiderTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::IRON_GOLEM,
                )),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SpiderEntity {}

impl Mob for SpiderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
