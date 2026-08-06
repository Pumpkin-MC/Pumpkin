//! Port of `Shoot.java`: charges up, then fires a `BreezeWindCharge` at the target.

use std::sync::Weak;
use std::sync::atomic::AtomicBool;

use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Controls, Goal, GoalFuture},
    mob::{Mob, breeze::BreezeEntity},
    projectile::ThrownItemEntity,
    projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity},
};

// Shoot.java constants.
const ATTACK_RANGE_MAX_SQR: f64 = 256.0;
const SHOOT_INITIAL_DELAY_TICKS: i32 = 15;
const SHOOT_RECOVER_DELAY_TICKS: i32 = 4;
const SHOOT_COOLDOWN_TICKS: i32 = 10;
const PROJECTILE_MOVEMENT_SCALE: f64 = 0.7;

pub struct BreezeShootGoal {
    breeze: Weak<BreezeEntity>,
    elapsed_ticks: i32,
    fired: bool,
}

impl BreezeShootGoal {
    #[must_use]
    pub const fn new(breeze: Weak<BreezeEntity>) -> Self {
        Self {
            breeze,
            elapsed_ticks: 0,
            fired: false,
        }
    }

    #[must_use]
    const fn is_target_within_range(distance_sqr: f64) -> bool {
        distance_sqr < ATTACK_RANGE_MAX_SQR
    }

    async fn fire(breeze: &BreezeEntity, target: &dyn EntityBase) {
        let shooter = &breeze.mob_entity.living_entity.entity;
        let world = shooter.world.load_full();
        let firing_pos = Vector3::new(
            shooter.pos.load().x,
            breeze.firing_y_position(),
            shooter.pos.load().z,
        );

        let charge_entity = Entity::new(world.clone(), firing_pos, &EntityType::BREEZE_WIND_CHARGE);
        let thrown = ThrownItemEntity {
            entity: charge_entity,
            owner_id: Some(shooter.entity_id),
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: WIND_CHARGE_GRAVITY,
        };
        let charge = WindChargeEntity::new_breeze(thrown);

        // `target.getY(passenger ? 0.8 : 0.3)` == `y + bbHeight * fraction`.
        let target_entity = target.get_entity();
        let target_y_fraction: f64 = if target_entity.has_vehicle().await {
            0.8
        } else {
            0.3
        };
        let target_pos = target_entity.pos.load();
        let target_y = target_y_fraction.mul_add(f64::from(target_entity.height()), target_pos.y);

        let dx = target_pos.x - firing_pos.x;
        let dy = target_y - firing_pos.y;
        let dz = target_pos.z - firing_pos.z;

        // Difficulty inaccuracy: `5 - difficulty.getId() * 4`.
        let difficulty = world.level_info.load().difficulty;
        let uncertainty = f64::from(5 - (difficulty as i32) * 4);
        charge.set_velocity(dx, dy, dz, PROJECTILE_MOVEMENT_SCALE, uncertainty);

        world.spawn_entity(std::sync::Arc::new(charge)).await;
        world.play_sound(
            Sound::EntityBreezeShoot,
            SoundCategory::Hostile,
            &firing_pos,
        );
    }
}

impl Goal for BreezeShootGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(breeze) = self.breeze.upgrade() else {
                return false;
            };
            // `BREEZE_SHOOT` present + no cooldown, mirroring `Shoot.java`'s memory
            // requirements; the window is opened by `BreezeJumpGoal` on landing.
            if breeze.shoot_window_ticks() <= 0 || breeze.shoot_cooldown_ticks() > 0 {
                return false;
            }
            let Some(target) = breeze.mob_entity.target.lock().await.clone() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }

            let distance_sqr = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());
            if !Self::is_target_within_range(distance_sqr) {
                breeze.set_shoot_window(0);
                return false;
            }

            true
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(breeze) = self.breeze.upgrade() else {
                return false;
            };
            self.elapsed_ticks < SHOOT_INITIAL_DELAY_TICKS + 1 + SHOOT_RECOVER_DELAY_TICKS
                && breeze.mob_entity.target.lock().await.is_some()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.elapsed_ticks = 0;
            self.fired = false;
            let entity = mob.get_entity();
            entity.set_pose(EntityPose::Shooting);
            let pos = entity.pos.load();
            entity
                .world
                .load()
                .play_sound(Sound::EntityBreezeInhale, SoundCategory::Hostile, &pos);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            if entity.pose.load() == EntityPose::Shooting {
                entity.set_pose(EntityPose::Standing);
            }
            if let Some(breeze) = self.breeze.upgrade() {
                breeze.set_shoot_cooldown(SHOOT_COOLDOWN_TICKS);
                breeze.set_shoot_window(0);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(breeze) = self.breeze.upgrade() else {
                return;
            };
            let Some(target) = breeze.mob_entity.target.lock().await.clone() else {
                return;
            };

            self.elapsed_ticks += 1;
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity(mob, &target);

            // Fires the tick after the `SHOOT_INITIAL_DELAY_TICKS`-tick charge-up ends.
            if !self.fired && self.elapsed_ticks == SHOOT_INITIAL_DELAY_TICKS + 1 {
                self.fired = true;
                Self::fire(&breeze, target.as_ref()).await;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

#[cfg(test)]
mod tests {
    use super::BreezeShootGoal;

    #[test]
    fn range_gate_matches_vanilla_sixteen_block_threshold() {
        assert!(BreezeShootGoal::is_target_within_range(255.99));
        assert!(!BreezeShootGoal::is_target_within_range(256.0));
    }
}
