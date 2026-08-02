use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
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
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
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
