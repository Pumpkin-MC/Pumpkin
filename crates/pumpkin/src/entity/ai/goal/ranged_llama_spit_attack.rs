use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::llama::llama_data_of;
use crate::entity::projectile::llama_spit::LlamaSpitEntity;
use crate::entity::{Entity, EntityBase};

/// `RangedAttackGoal` as instantiated by `Llama`.
///
/// (`RangedAttackGoal(this, 1.25, 40, 20.0F)`,
/// `Llama.java:121`). Vanilla's `RangedAttackGoal` is generic over any `RangedAttackMob`; this
/// port is specialized to the llama spit (the only ranged-attack mob wired up so far that would
/// use it), the same specialize-on-first-user approach `RangedSnowballAttackGoal` already takes.
pub struct RangedLlamaSpitAttackGoal {
    speed: f64,
    attack_interval_min: i32,
    attack_interval_max: i32,
    attack_radius: f32,
    target: Option<Arc<dyn EntityBase>>,
    attack_time: i32,
    see_time: i32,
}

impl RangedLlamaSpitAttackGoal {
    #[must_use]
    pub fn new(speed: f64, attack_interval: i32, attack_radius: f32) -> Box<Self> {
        Box::new(Self {
            speed,
            attack_interval_min: attack_interval,
            attack_interval_max: attack_interval,
            attack_radius,
            target: None,
            attack_time: -1,
            see_time: 0,
        })
    }

    /// `Llama.spit` (`Llama.java:340-365`): aims from the shooter's near-eye position toward
    /// `target.getY(1/3)` with a horizontal-distance-scaled vertical lead, shot at speed 1.5.
    /// Vanilla additionally applies a small random inaccuracy spread (`spawnProjectileUsingShoot`
    /// inaccuracy 10); Pumpkin has no equivalent helper, so the direction is normalized and scaled
    /// exactly, without the random spread -- documented simplification.
    fn spit_velocity(shooter: Vector3<f64>, target: Vector3<f64>) -> Vector3<f64> {
        let dx = target.x - shooter.x;
        let dz = target.z - shooter.z;
        let horizontal = dx.hypot(dz);
        let lead = horizontal * 0.2;
        let dy = target.y - shooter.y + lead;
        Vector3::new(dx, dy, dz).normalize() * 1.5
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let entity = mob.get_entity();
        let target_entity = target.get_entity();
        let from = entity.get_eye_pos();
        let to = target_entity.get_eye_pos();
        if from.squared_distance_to_vec(&to) > 128.0 * 128.0 {
            return false;
        }

        entity
            .world
            .load_full()
            .raycast_collision(from, to, async |block_pos, world| {
                !world.get_block_state(block_pos).collision_shapes.is_empty()
            })
            .await
            .is_none()
    }

    async fn shoot(&self, mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let shooter_pos = shooter.pos.load();
        let spawn_pos = Vector3::new(shooter_pos.x, shooter.get_eye_y() - 0.1, shooter_pos.z);

        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_y_third = target_pos.y + f64::from(target_entity.height()) / 3.0;

        let projectile_entity = Entity::new(world.clone(), spawn_pos, &EntityType::LLAMA_SPIT);
        let spit = LlamaSpitEntity::new_shot(projectile_entity, shooter);

        let velocity = Self::spit_velocity(
            spawn_pos,
            Vector3::new(target_pos.x, target_y_third, target_pos.z),
        );
        spit.get_entity().set_velocity(velocity);
        world.spawn_entity(Arc::new(spit)).await;
        world.play_sound(Sound::EntityLlamaSpit, SoundCategory::Neutral, &shooter_pos);

        if let Some(data) = llama_data_of(mob as &dyn EntityBase) {
            data.did_spit
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl Goal for RangedLlamaSpitAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();
            match target {
                Some(target) if target.get_entity().is_alive() => {
                    self.target = Some(target);
                    true
                }
                _ => false,
            }
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(target) = self.target.clone() else {
                return false;
            };
            if target.get_entity().is_alive() {
                return true;
            }
            !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            self.see_time = 0;
            self.attack_time = -1;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = self.target.clone() else {
                return;
            };

            let mob_entity = mob.get_mob_entity();
            let shooter_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sqr = shooter_pos.squared_distance_to_vec(&target_pos);

            let has_line_of_sight = Self::has_line_of_sight(mob, target.as_ref()).await;
            if has_line_of_sight {
                self.see_time += 1;
            } else {
                self.see_time = 0;
            }

            let in_range = dist_sqr <= f64::from(self.attack_radius * self.attack_radius);
            if in_range && self.see_time >= 5 {
                mob_entity.navigator.lock().unwrap().stop();
            } else {
                mob_entity
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal::new(shooter_pos, target_pos, self.speed));
            }

            mob_entity
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            self.attack_time -= 1;
            if self.attack_time == 0 {
                if !has_line_of_sight {
                    return;
                }
                let dist_ratio = (dist_sqr.sqrt() as f32 / self.attack_radius).clamp(0.1, 1.0);
                self.shoot(mob, target.as_ref()).await;
                self.attack_time = (dist_ratio
                    * (self.attack_interval_max - self.attack_interval_min) as f32
                    + self.attack_interval_min as f32) as i32;
            } else if self.attack_time < 0 {
                let t = (dist_sqr.sqrt() / f64::from(self.attack_radius)).clamp(0.0, 1.0);
                self.attack_time = (f64::from(self.attack_interval_min)
                    + t * f64::from(self.attack_interval_max - self.attack_interval_min))
                    as i32;
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
    use super::RangedLlamaSpitAttackGoal;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn spit_velocity_is_scaled_to_vanilla_speed() {
        let velocity = RangedLlamaSpitAttackGoal::spit_velocity(
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(3.0, 1.0, 4.0),
        );
        assert!((velocity.length() - 1.5).abs() < 1.0e-9);
        // Level target, positive horizontal distance -> lead pushes the shot upward.
        assert!(velocity.y > 0.0);
    }

    #[test]
    fn spit_velocity_at_same_position_has_no_horizontal_component() {
        let velocity = RangedLlamaSpitAttackGoal::spit_velocity(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 5.0, 0.0),
        );
        assert_eq!(velocity.x, 0.0);
        assert_eq!(velocity.z, 0.0);
        assert!((velocity.y - 1.5).abs() < 1.0e-9);
    }
}
