use std::sync::atomic::{AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, breeze_jump::BreezeJumpGoal,
        breeze_shoot::BreezeShootGoal, breeze_slide::BreezeSlideGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct BreezeEntity {
    pub mob_entity: MobEntity,
    /// Mirrors vanilla's `BREEZE_SHOOT` memory: ticks remaining in the shoot-permission
    /// window opened by `BreezeJumpGoal` on landing. `Shoot.java` requires it present to
    /// start; `LongJump.java` requires it absent, so this token also keeps the two goals
    /// mutually exclusive.
    shoot_window_ticks: AtomicI32,
    /// Mirrors `BREEZE_SHOOT_COOLDOWN`, set after `Shoot.java` stops.
    shoot_cooldown_ticks: AtomicI32,
    /// Mirrors `BREEZE_JUMP_COOLDOWN`.
    jump_cooldown_ticks: AtomicI32,
}

impl BreezeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let breeze = Self {
            mob_entity,
            shoot_window_ticks: AtomicI32::new(0),
            shoot_cooldown_ticks: AtomicI32::new(0),
            jump_cooldown_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(breeze);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let breeze_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Priorities mirror BreezeAi's FIGHT activity ordering (Shoot, LongJump, Slide).
            // `ShootWhenStuck` (passenger/water/levitation fallback) is skipped: it only
            // matters for edge cases Pumpkin doesn't yet model (riding, levitation).
            goal_selector.add_goal(1, Box::new(BreezeShootGoal::new(breeze_weak.clone())));
            goal_selector.add_goal(2, Box::new(BreezeJumpGoal::new(breeze_weak.clone())));
            goal_selector.add_goal(3, Box::new(BreezeSlideGoal::new(breeze_weak)));
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
        };

        mob_arc
    }

    fn decrement(counter: &AtomicI32) {
        let value = counter.load(Relaxed);
        if value > 0 {
            counter.store(value - 1, Relaxed);
        }
    }

    #[must_use]
    pub fn shoot_window_ticks(&self) -> i32 {
        self.shoot_window_ticks.load(Relaxed)
    }

    pub fn set_shoot_window(&self, ticks: i32) {
        self.shoot_window_ticks.store(ticks, Relaxed);
    }

    #[must_use]
    pub fn shoot_cooldown_ticks(&self) -> i32 {
        self.shoot_cooldown_ticks.load(Relaxed)
    }

    pub fn set_shoot_cooldown(&self, ticks: i32) {
        self.shoot_cooldown_ticks.store(ticks, Relaxed);
    }

    #[must_use]
    pub fn jump_cooldown_ticks(&self) -> i32 {
        self.jump_cooldown_ticks.load(Relaxed)
    }

    pub fn set_jump_cooldown(&self, ticks: i32) {
        self.jump_cooldown_ticks.store(ticks, Relaxed);
    }

    /// `Breeze.getFiringYPosition`: `y + bbHeight / 2 + 0.3`.
    #[must_use]
    pub fn firing_y_position(&self) -> f64 {
        let entity = &self.mob_entity.living_entity.entity;
        entity.pos.load().y + f64::from(entity.height()) / 2.0 + 0.3
    }
}

impl NBTStorage for BreezeEntity {}

impl Mob for BreezeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            Self::decrement(&self.shoot_window_ticks);
            Self::decrement(&self.shoot_cooldown_ticks);
            Self::decrement(&self.jump_cooldown_ticks);
        })
    }
}
