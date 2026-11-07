use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::ai::util::RandomExt;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const BLAZE_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true)];
const INITIAL_ALLOWED_HEIGHT_OFFSET: f32 = 0.5;
const HEIGHT_OFFSET_CHANGE_INTERVAL: i32 = 100;
const HEIGHT_OFFSET_SPREAD: f64 = 6.891;
const ASCENT_ACCELERATION: f64 = 0.3f32 as f64;

pub struct BlazeEntity {
    pub entity: Arc<MobEntity>,
    pub is_charged: AtomicBool,
    allowed_height_offset: AtomicCell<f32>,
    next_height_offset_change_tick: AtomicI32,
}

impl BlazeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let blaze = Self {
            entity,
            is_charged: AtomicBool::new(false),
            allowed_height_offset: AtomicCell::new(INITIAL_ALLOWED_HEIGHT_OFFSET),
            next_height_offset_change_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(blaze);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let goal_selector = &mob_arc.entity.goals_selector;
            let target_selector = &mob_arc.entity.target_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));

            goal_selector.add_goal(
                4,
                Box::new(
                    crate::entity::ai::goal::blaze_attack::BlazeShootFireballGoal::new(
                        Arc::downgrade(&mob_arc),
                    ),
                ),
            );

            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
            hunting::apply(target_selector, &mob_arc.entity, BLAZE_HUNTS);
        };

        mob_arc
    }

    pub fn is_charged(&self) -> bool {
        self.is_charged.load(Ordering::Relaxed)
    }

    pub fn set_charged(&self, charged: bool) {
        self.is_charged.store(charged, Ordering::Relaxed);
        let flags = i8::from(charged);
        self.entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::blaze::DATA_FLAGS_ID, flags);
    }
}

impl Mob for BlazeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn additional_tick_movement(&self) -> f64 {
        // Ascent contracts the old vertical velocity and adds this much from rest.
        ASCENT_ACCELERATION * ASCENT_ACCELERATION
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let base_entity = &self.entity.living_entity.entity;
        if !base_entity.is_alive() {
            return;
        }

        let on_ground = base_entity.on_ground.load(Ordering::Relaxed);
        let vel = base_entity.velocity.load();
        if !on_ground && vel.y < 0.0 {
            base_entity
                .velocity
                .store(Vector3::new(vel.x, vel.y * 0.6, vel.z));
        }

        if base_entity.touching_water.load(Ordering::Relaxed) {
            caller.damage(caller, 1.0, DamageType::DROWN);
        }
    }

    /// Vanilla `Blaze.customServerAiStep`: lifts toward a target above the height offset.
    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        if self
            .next_height_offset_change_tick
            .fetch_sub(1, Ordering::Relaxed)
            <= 1
        {
            self.next_height_offset_change_tick
                .store(HEIGHT_OFFSET_CHANGE_INTERVAL, Ordering::Relaxed);
            let mut rng = self.get_random();
            self.allowed_height_offset.store(rng.triangle(
                f64::from(INITIAL_ALLOWED_HEIGHT_OFFSET),
                HEIGHT_OFFSET_SPREAD,
            ) as f32);
        }

        let Some(target) = self.entity.get_target() else {
            return;
        };
        let entity = &self.entity.living_entity.entity;
        if target.get_entity().get_eye_y()
            > entity.get_eye_y() + f64::from(self.allowed_height_offset.load())
            && self.can_attack(target.as_ref())
        {
            let velocity = entity.velocity.load();
            entity.velocity.store(Vector3::new(
                velocity.x,
                velocity.y + (ASCENT_ACCELERATION - velocity.y) * ASCENT_ACCELERATION,
                velocity.z,
            ));
            entity.needs_sync.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;

    /// Ascent needs a strictly higher, attackable target and stops once the target is lost.
    #[tokio::test]
    async fn ascent_requires_a_higher_attackable_target_and_stops_after_target_loss()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let blaze = BlazeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BLAZE,
        ));
        let target = BlazeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 65.0, 0.0),
            &EntityType::BLAZE,
        ));
        let entity = blaze.get_entity();
        blaze.entity.set_target(Some(target.clone()));
        // Keep the threshold fixed while testing the strict eye-height boundary.
        blaze
            .next_height_offset_change_tick
            .store(20, Ordering::Relaxed);
        blaze.allowed_height_offset.store(1.0);
        let velocity = Vector3::new(0.1, 0.0, -0.1);
        entity.velocity.store(velocity);
        entity.velocity_dirty.store(false, Ordering::Relaxed);
        blaze.custom_server_ai_step(blaze.as_ref());
        assert_eq!(entity.velocity.load(), velocity);
        assert!(!entity.needs_sync.load(Ordering::Relaxed));

        target.get_entity().set_pos(Vector3::new(0.0, 66.0, 0.0));
        blaze.custom_server_ai_step(blaze.as_ref());
        let ascending = entity.velocity.load();
        // Official 26.3's first ascent impulse from rest.
        assert!((ascending.y - 0.090_000_007_152_557_52).abs() < f64::EPSILON);
        assert_eq!((ascending.x, ascending.z), (velocity.x, velocity.z));
        assert!(entity.needs_sync.swap(false, Ordering::Relaxed));
        assert!(!entity.velocity_dirty.load(Ordering::Relaxed));

        target.get_entity().set_invulnerable(true);
        blaze.custom_server_ai_step(blaze.as_ref());
        assert_eq!(entity.velocity.load(), ascending);
        assert!(!entity.needs_sync.load(Ordering::Relaxed));

        blaze.entity.set_target(None);
        blaze.custom_server_ai_step(blaze.as_ref());
        assert_eq!(entity.velocity.load(), ascending);
        assert!(!entity.needs_sync.load(Ordering::Relaxed));
        world.level.shutdown().await;
        Ok(())
    }

    /// The height offset refreshes on the first AI step and then every 100 steps.
    #[tokio::test]
    async fn height_offset_refreshes_first_step_then_every_100_ai_steps()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let blaze = BlazeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BLAZE,
        ));
        // Outside the triangle(0.5, 6.891) range, so any refresh replaces it.
        let sentinel = 100.0;
        let in_range = |offset: f32| {
            (f64::from(offset) - f64::from(INITIAL_ALLOWED_HEIGHT_OFFSET)).abs()
                <= HEIGHT_OFFSET_SPREAD
        };

        blaze.allowed_height_offset.store(sentinel);
        blaze.custom_server_ai_step(blaze.as_ref());
        assert!(in_range(blaze.allowed_height_offset.load()));
        assert_eq!(
            blaze.next_height_offset_change_tick.load(Ordering::Relaxed),
            HEIGHT_OFFSET_CHANGE_INTERVAL
        );

        blaze.allowed_height_offset.store(sentinel);
        for _ in 1..HEIGHT_OFFSET_CHANGE_INTERVAL {
            blaze.custom_server_ai_step(blaze.as_ref());
        }
        assert_eq!(blaze.allowed_height_offset.load(), sentinel);
        blaze.custom_server_ai_step(blaze.as_ref());
        assert!(in_range(blaze.allowed_height_offset.load()));
        assert_eq!(
            blaze.next_height_offset_change_tick.load(Ordering::Relaxed),
            HEIGHT_OFFSET_CHANGE_INTERVAL
        );
        world.level.shutdown().await;
        Ok(())
    }
}
