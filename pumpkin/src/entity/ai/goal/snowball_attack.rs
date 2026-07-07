use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::entity::{Entity, EntityBase};
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use std::sync::Arc;

/// Ranged snowball-throwing behavior for the snow golem.
///
/// Mirrors vanilla `ProjectileAttackGoal`: the golem approaches until the target
/// is in range, holds position, and lobs a snowball on a fixed interval. Mob
/// line of sight isn't modelled upstream yet (see `BlazeShootFireballGoal`), so
/// the target counts as visible while the goal runs.
pub struct SnowballAttackGoal {
    /// Movement speed multiplier while approaching the target.
    speed: f64,
    /// Squared maximum throwing range.
    squared_range: f64,
    /// Ticks between two throws.
    interval: i32,
    /// Ticks until the next throw (`<= 0` means ready).
    cooldown: i32,
    /// How long the target has been continuously visible.
    target_seeing_ticks: i32,
}

impl SnowballAttackGoal {
    /// `speed` is the approach speed multiplier, `interval` the ticks between
    /// throws, and `range` the maximum throwing distance in blocks.
    #[must_use]
    pub fn new(speed: f64, interval: i32, range: f32) -> Box<Self> {
        Box::new(Self {
            speed,
            squared_range: f64::from(range * range),
            interval,
            cooldown: -1,
            target_seeing_ticks: 0,
        })
    }

    /// Lobs a snowball from the golem toward `target`, matching vanilla
    /// `SnowGolemEntity::shootAt` ballistics: aim at `eyeY - 1.1` and
    /// `velocity(dx, dy + horizontal * 0.2, dz, 1.6, 12.0)`.
    async fn shoot_at(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();

        let mut spawn_pos = shooter.pos.load();
        spawn_pos.y = shooter.get_eye_y() - 0.1;
        let snowball_entity = Entity::from_uuid(
            uuid::Uuid::new_v4(),
            world.clone(),
            spawn_pos,
            &EntityType::SNOWBALL,
        );
        let snowball = SnowballEntity::new_shot(snowball_entity, shooter);

        let shooter_pos = shooter.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();

        let dx = target_pos.x - shooter_pos.x;
        // Aim at the target's upper body, like vanilla's `getEyeY() - 1.1`.
        let dy = (target_entity.get_eye_y() - 1.1) - spawn_pos.y;
        let dz = target_pos.z - shooter_pos.z;
        let horizontal = dx.hypot(dz);

        snowball
            .thrown
            .set_velocity(dx, horizontal.mul_add(0.2, dy), dz, 1.6, 12.0);

        world.play_sound(
            Sound::EntitySnowGolemShoot,
            SoundCategory::Neutral,
            &shooter_pos,
        );
        world.spawn_entity(Arc::new(snowball)).await;
    }
}

impl Goal for SnowballAttackGoal {
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

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.get_mob_entity().set_attacking(false);
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            self.cooldown = -1;
            self.target_seeing_ticks = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_sq = mob_pos.squared_distance_to_vec(&target_pos);

            // Upstream has no mob line-of-sight raycast yet, so the target counts
            // as continuously visible while this goal runs.
            self.target_seeing_ticks += 1;

            let in_range = distance_sq <= self.squared_range && self.target_seeing_ticks >= 5;
            {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                if in_range {
                    navigator.stop();
                } else {
                    navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, self.speed));
                }
            }

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);

            if in_range {
                if self.cooldown > 0 {
                    self.cooldown -= 1;
                } else {
                    Self::shoot_at(mob, target.as_ref()).await;
                    self.cooldown = self.interval;
                }
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
