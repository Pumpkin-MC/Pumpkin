use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, move_towards_restriction::MoveTowardsRestrictionGoal,
        revenge::RevengeGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

pub struct BlazeEntity {
    pub entity: Arc<MobEntity>,
    /// Vanilla `Blaze.allowedHeightOffset` (`Blaze.java:26`).
    allowed_height_offset: AtomicCell<f32>,
    /// Vanilla `Blaze.nextHeightOffsetChangeTick` (`Blaze.java:27`).
    next_height_offset_change_tick: AtomicI32,
    /// Vanilla `Blaze.DATA_FLAGS_ID` bit 0 (`Blaze.java:32,138-153`).
    charged: AtomicBool,
}

impl BlazeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let zombie = Self {
            entity,
            allowed_height_offset: AtomicCell::new(0.5),
            next_height_offset_change_tick: AtomicI32::new(0),
            charged: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let mut navigator = mob_arc.entity.navigator.lock().unwrap();
            // `Blaze.<init>` (`Blaze.java:42-44`). The Rust path evaluator names
            // `FIRE_IN_NEIGHBOR`/`FIRE` as `DangerFire`/`DamageFire`.
            navigator.set_pathfinding_malus(PathType::Water, -1.0);
            navigator.set_pathfinding_malus(PathType::Lava, 8.0);
            navigator.set_pathfinding_malus(PathType::DangerFire, 0.0);
            navigator.set_pathfinding_malus(PathType::DamageFire, 0.0);
            drop(navigator);

            let mut goal_selector = mob_arc.entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));

            goal_selector.add_goal(
                4,
                Box::new(
                    crate::entity::ai::goal::blaze_attack::BlazeShootFireballGoal::new(
                        Arc::downgrade(&mob_arc),
                    ),
                ),
            );

            goal_selector.add_goal(5, MoveTowardsRestrictionGoal::new(1.0));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new_water_avoiding(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub async fn set_charged(&self, charged: bool) {
        self.charged.store(charged, Relaxed);
        self.entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BLAZE_FLAGS,
                MetaDataType::BYTE,
                i8::from(charged),
            )],
            None,
        );
        // `BLAZE_FLAGS` is absent from the generated v26.x protocol mapping. Keep the
        // same client-visible fire state on those versions while retaining the vanilla
        // Blaze-specific flag for versions where it is present.
        self.entity.living_entity.entity.set_on_fire(charged).await;
    }

    #[must_use]
    pub fn is_charged(&self) -> bool {
        self.charged.load(Relaxed)
    }

    fn update_height_offset(&self) {
        // Java's `nextHeightOffsetChangeTick-- <= 0` tests the old value.
        let previous = self.next_height_offset_change_tick.fetch_sub(1, Relaxed);
        if previous <= 0 {
            let mut random = rand::rng();
            self.next_height_offset_change_tick.store(100, Relaxed);
            self.allowed_height_offset
                .store(triangle(&mut random, 0.5, 6.891) as f32);
        }
    }

    #[must_use]
    fn allowed_height_offset(&self) -> f64 {
        f64::from(self.allowed_height_offset.load())
    }
}

impl NBTStorage for BlazeEntity {}

impl Mob for BlazeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity.living_entity.entity;

            // `Blaze.aiStep` (`Blaze.java:95-101`): only damp downward motion while
            // airborne. This runs before LivingEntity's movement integration below.
            if !entity.on_ground.load(Relaxed) {
                let velocity = entity.velocity.load();
                if velocity.y < 0.0 {
                    entity.set_velocity(velocity.multiply(1.0, 0.6, 1.0));
                }
            }

            // `Blaze.customServerAiStep` (`Blaze.java:119-136`). The target-height
            // adjustment is applied before the regular goal/navigation tick, matching
            // the vanilla call order.
            self.update_height_offset();
            let target = self.entity.target.lock().await.clone();
            if let Some(target) = target
                && target.get_entity().get_eye_y()
                    > entity.get_eye_y() + self.allowed_height_offset()
                && self.can_attack(target.get_entity())
            {
                let velocity = entity.velocity.load();
                entity.add_velocity(Vector3::new(0.0, (0.3 - velocity.y) * 0.3, 0.0));
            }
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::BLAZE_FLAGS,
                    MetaDataType::BYTE,
                    i8::from(self.is_charged()),
                )],
                None,
            );
        })
    }
}

/// `RandomSource.triangle(mean, spread)` (`RandomSource.java:59-61`).
fn triangle<R: rand::Rng + ?Sized>(random: &mut R, mean: f64, spread: f64) -> f64 {
    mean + spread * (random.random::<f64>() - random.random::<f64>())
}

#[cfg(test)]
mod tests {
    use super::triangle;

    #[test]
    fn triangle_uses_vanilla_mean_plus_difference_formula() {
        let mut random = rand::rng();
        let value = triangle(&mut random, 0.5, 6.891);
        assert!((-6.391..=7.391).contains(&value));
    }
}
