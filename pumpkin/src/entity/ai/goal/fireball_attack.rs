//! Ghast large fireball attack (vanilla `GhastShootFireballGoal` simplified).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::projectile::fireball::FireballEntity;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

const ATTACK_RADIUS_SQ: f64 = 64.0 * 64.0;
const CHARGE_TICKS: i32 = 20;
const COOLDOWN: i32 = 40;

pub struct FireballAttackGoal {
    /// >0 charging, 0 idle, <0 cooldown.
    charge_time: i32,
}

impl FireballAttackGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { charge_time: 0 })
    }

    fn in_range(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        (target_pos.y - mob_pos.y).abs() <= 4.0
            && mob_pos.squared_distance_to_vec(&target_pos) < ATTACK_RADIUS_SQ
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

        let entity = Entity::new(world.clone(), eye, &EntityType::FIREBALL);
        let ball = FireballEntity::new_shot(entity, shooter);
        let (yaw, pitch) = Self::look_angles(eye, target_eye);
        ball.thrown
            .set_velocity_from(shooter, pitch, yaw, 0.0, 1.0, 0.0);

        world.spawn_entity(Arc::new(ball)).await;
        world.play_sound(
            Sound::EntityGhastShoot,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
    }
}

impl Goal for FireballAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target
                .as_ref()
                .is_some_and(|t| t.get_entity().is_alive() && Self::in_range(mob, t.as_ref()))
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target
                .as_ref()
                .is_some_and(|t| t.get_entity().is_alive() && Self::in_range(mob, t.as_ref()))
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

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 10.0, 10.0);
            }

            if self.charge_time < 0 {
                self.charge_time += 1;
                return;
            }

            if self.charge_time == 0 {
                self.charge_time = 1;
                let entity = mob.get_entity();
                let world = entity.world.load();
                world.play_sound(
                    Sound::EntityGhastWarn,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                );
                return;
            }

            self.charge_time += 1;
            if self.charge_time >= to_goal_ticks(CHARGE_TICKS) {
                Self::shoot(mob, target.as_ref()).await;
                self.charge_time = -to_goal_ticks(COOLDOWN);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}
