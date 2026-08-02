use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::{Entity, EntityBase};

pub struct RangedSnowballAttackGoal {
    cooldown: i32,
    interval: i32,
    range: f64,
}

impl RangedSnowballAttackGoal {
    #[must_use]
    pub const fn new(interval: i32, range: f64) -> Self {
        Self {
            cooldown: 0,
            interval,
            range,
        }
    }

    fn projectile_velocity(
        shooter: Vector3<f64>,
        target: Vector3<f64>,
        target_eye_height: f64,
    ) -> Vector3<f64> {
        let x = target.x - shooter.x;
        let z = target.z - shooter.z;
        let horizontal = x.hypot(z);
        let y = target.y + target_eye_height - 1.1 - shooter.y + horizontal * 0.2;
        Vector3::new(x, y, z).normalize() * 1.6
    }

    async fn shoot(&self, mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let position = shooter.pos.load();
        let projectile_entity = Entity::new(world.clone(), position, &EntityType::SNOWBALL);
        let projectile = SnowballEntity::new_shot(projectile_entity, shooter);
        let velocity = Self::projectile_velocity(
            position,
            target.get_entity().pos.load(),
            target.get_entity().get_eye_height(),
        );
        projectile.get_entity().set_velocity(velocity);
        world.spawn_entity(Arc::new(projectile)).await;
        world.play_sound(
            Sound::EntitySnowGolemShoot,
            SoundCategory::Hostile,
            &position,
        );
    }
}

impl Goal for RangedSnowballAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            mob.get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            mob.get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.cooldown = 0;
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            mob.get_mob_entity().set_attacking(false);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            let shooter = mob.get_entity();
            let shooter_pos = shooter.pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_squared = shooter_pos.squared_distance_to_vec(&target_pos);

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            self.cooldown = (self.cooldown - 1).max(0);

            if distance_squared > self.range * self.range {
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal {
                        current_progress: shooter_pos,
                        destination: target_pos,
                        speed: 1.0,
                    });
                return;
            }

            mob.get_mob_entity().navigator.lock().unwrap().stop();
            if self.cooldown == 0 {
                self.shoot(mob, target.as_ref()).await;
                self.cooldown = self.interval;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::RangedSnowballAttackGoal;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn snowball_velocity_uses_vanilla_vertical_lead() {
        let velocity = RangedSnowballAttackGoal::projectile_velocity(
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(3.0, 0.0, 4.0),
            1.8,
        );
        assert!(velocity.y > 0.0);
        assert!((velocity.length() - 1.6).abs() < 1.0e-9);
    }

    #[test]
    fn snowball_goal_uses_vanilla_interval_and_range() {
        let goal = RangedSnowballAttackGoal::new(20, 10.0);
        assert_eq!(goal.interval, 20);
        assert_eq!(goal.range, 10.0);
    }
}
