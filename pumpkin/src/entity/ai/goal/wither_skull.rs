//! Wither skull ranged attack stand-in (uses fireball physics + wither shoot sound).

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::projectile::fireball::FireballEntity;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

const RANGE_SQ: f64 = 40.0 * 40.0;
const INTERVAL: i32 = 40;

pub struct WitherSkullGoal {
    cooldown: i32,
}

impl WitherSkullGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { cooldown: 0 })
    }

    fn look_angles(from: Vector3<f64>, to: Vector3<f64>) -> (f32, f32) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;
        let horiz = (dx * dx + dz * dz).sqrt();
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(horiz).to_degrees() as f32);
        (yaw, pitch)
    }

    async fn shoot(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let eye = shooter.get_eye_pos();
        let target_eye = target.get_entity().get_eye_pos();

        // WITHER_SKULL type for correct client entity; fireball motion/explosion.
        let entity = Entity::new(world.clone(), eye, &EntityType::WITHER_SKULL);
        let skull = FireballEntity::new_shot(entity, shooter);
        let (yaw, pitch) = Self::look_angles(eye, target_eye);
        skull
            .thrown
            .set_velocity_from(shooter, pitch, yaw, 0.0, 1.0, 0.0);

        world.spawn_entity(Arc::new(skull)).await;
        world.play_sound(
            Sound::EntityWitherShoot,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
    }
}

impl Goal for WitherSkullGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| {
                t.get_entity().is_alive()
                    && mob
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&t.get_entity().pos.load())
                        < RANGE_SQ
            })
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

            {
                let eye = target.get_entity().get_eye_pos();
                let mut look = mob.get_mob_entity().look_control.lock().unwrap();
                look.look_at_with_range(eye.x, eye.y, eye.z, 40.0, 40.0);
            }

            self.cooldown -= 1;
            if self.cooldown <= 0 {
                Self::shoot(mob, target.as_ref()).await;
                self.cooldown = to_goal_ticks(INTERVAL);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}
