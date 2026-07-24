//! Breeze wind-charge ranged attack (simplified).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::ThrownItemEntity;
use crate::entity::projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity};
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

const ATTACK_RADIUS_SQ: f64 = 16.0 * 16.0;
const THROW_INTERVAL: i32 = 40;

pub struct WindChargeAttackGoal {
    attack_time: i32,
    speed: f64,
}

impl WindChargeAttackGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            attack_time: 0,
            speed: speed.max(0.25),
        })
    }

    fn look_angles(from: Vector3<f64>, to: Vector3<f64>) -> (f32, f32) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;
        let horiz = dx.hypot(dz);
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(horiz).to_degrees() as f32);
        (yaw, pitch)
    }

    async fn shoot(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let eye = shooter.get_eye_pos();
        let target_eye = target.get_entity().get_eye_pos();

        let entity = Entity::new(world.clone(), eye, &EntityType::BREEZE_WIND_CHARGE);
        let thrown = ThrownItemEntity::new(entity, shooter, WIND_CHARGE_GRAVITY);
        let charge = WindChargeEntity::new_breeze(thrown);

        let (yaw, pitch) = Self::look_angles(eye, target_eye);
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let vx = f64::from(-yaw_rad.sin() * pitch_rad.cos());
        let vy = f64::from(-pitch_rad.sin());
        let vz = f64::from(yaw_rad.cos() * pitch_rad.cos());
        let vel = Vector3::new(vx, vy, vz).normalize().multiply(1.4, 1.4, 1.4);
        charge.get_entity().set_velocity(vel);

        world.spawn_entity(Arc::new(charge)).await;
        world.play_sound(
            Sound::EntityBreezeShoot,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
        mob.get_mob_entity().living_entity.swing_hand().await;
    }
}

impl Goal for WindChargeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 30.0, 30.0);
            }

            if dist_sq > ATTACK_RADIUS_SQ {
                let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
                nav.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
            } else {
                mob.get_mob_entity().navigator.lock().unwrap().stop();
            }

            self.attack_time -= 1;
            if self.attack_time <= 0 && dist_sq <= ATTACK_RADIUS_SQ {
                Self::shoot(mob, target.as_ref()).await;
                self.attack_time = to_goal_ticks(THROW_INTERVAL);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
