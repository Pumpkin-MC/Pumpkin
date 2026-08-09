use std::sync::Arc;

use pumpkin_data::{
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::{Difficulty, math::vector3::Vector3};

use crate::entity::{
    Entity, EntityBase,
    ai::{
        goal::{Controls, Goal, GoalFuture},
        pathfinder::NavigatorGoal,
    },
    mob::Mob,
    predicate::EntityPredicate,
    projectile::arrow::{ArrowEntity, ArrowPickup},
};

pub struct BowAttackGoal {
    speed: f64,
    attack_cooldown: i32,
    max_range_squared: f64,
    last_target_position: Option<Vector3<f64>>,
}

impl BowAttackGoal {
    const BOW_DRAW_TIME: i32 = 20;

    #[must_use]
    pub fn new(speed: f64, max_range: f64) -> Self {
        Self {
            speed: speed.max(0.23),
            attack_cooldown: 0,
            max_range_squared: max_range * max_range,
            last_target_position: None,
        }
    }

    const fn attack_interval(difficulty: Difficulty) -> i32 {
        if matches!(difficulty, Difficulty::Hard) {
            20
        } else {
            40
        }
    }

    const fn divergence(difficulty: Difficulty) -> f64 {
        let difficulty_id = match difficulty {
            Difficulty::Peaceful => 0,
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
        };
        (14 - difficulty_id * 4) as f64
    }

    fn launch_direction(shooter: &Entity, target: &Entity) -> Vector3<f64> {
        let shooter_pos = shooter.pos.load();
        let target_pos = target.pos.load();
        let x = target_pos.x - shooter_pos.x;
        let z = target_pos.z - shooter_pos.z;
        let horizontal_distance = x.hypot(z);
        let target_y = target_pos.y + f64::from(target.entity_dimension.load().height) / 3.0;
        let y = target_y - (shooter.get_eye_y() - 0.1) + horizontal_distance * 0.2;
        Vector3::new(x, y, z)
    }

    async fn shoot(mob: &dyn Mob, target: &dyn EntityBase, difficulty: Difficulty) {
        let shooter = mob.get_entity();
        let world = shooter.world.load();
        let projectile = ItemStack::new(1, &Item::ARROW);
        let arrow_entity = Entity::new(
            world.clone(),
            shooter.pos.load(),
            ArrowEntity::entity_type_for_item(projectile.item),
        );
        let arrow =
            ArrowEntity::new_shot(arrow_entity, shooter, &projectile, ArrowPickup::Disallowed);
        let direction = Self::launch_direction(shooter, target.get_entity());
        arrow.set_velocity(
            direction.x,
            direction.y,
            direction.z,
            1.6,
            Self::divergence(difficulty),
        );

        world.spawn_entity(Arc::new(arrow)).await;
        world.play_sound(
            Sound::EntitySkeletonShoot,
            SoundCategory::Hostile,
            &shooter.pos.load(),
        );
    }
}

impl Goal for BowAttackGoal {
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
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            target.get_entity().is_alive()
                && !EntityPredicate::ExceptCreativeOrSpectator
                    .test(target.get_entity())
                    .await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_cooldown = Self::BOW_DRAW_TIME;
            self.last_target_position = None;
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
            mob.get_mob_entity().set_attacking(false);
            self.last_target_position = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity_with_range(&target, 30.0, 30.0);

            let shooter_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let distance_squared = shooter_pos.squared_distance_to_vec(&target_pos);

            if distance_squared > self.max_range_squared {
                if self.last_target_position != Some(target_pos) {
                    mob.get_mob_entity()
                        .navigator
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .set_progress(NavigatorGoal {
                            current_progress: shooter_pos,
                            destination: target_pos,
                            speed: self.speed,
                        });
                    self.last_target_position = Some(target_pos);
                }
            } else {
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .stop();
                self.last_target_position = None;
            }

            self.attack_cooldown = (self.attack_cooldown - 1).max(0);
            if distance_squared <= self.max_range_squared && self.attack_cooldown == 0 {
                let difficulty = mob.get_entity().world.load().level_info.load().difficulty;
                Self::shoot(mob, target.as_ref(), difficulty).await;
                self.attack_cooldown = Self::attack_interval(difficulty);
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
    use pumpkin_util::Difficulty;

    use super::BowAttackGoal;

    #[test]
    fn skeleton_bow_timing_scales_with_difficulty() {
        assert_eq!(BowAttackGoal::BOW_DRAW_TIME, 20);
        assert_eq!(BowAttackGoal::attack_interval(Difficulty::Easy), 40);
        assert_eq!(BowAttackGoal::attack_interval(Difficulty::Normal), 40);
        assert_eq!(BowAttackGoal::attack_interval(Difficulty::Hard), 20);
    }

    #[test]
    fn skeleton_arrows_get_more_accurate_on_higher_difficulties() {
        assert_eq!(BowAttackGoal::divergence(Difficulty::Easy), 10.0);
        assert_eq!(BowAttackGoal::divergence(Difficulty::Normal), 6.0);
        assert_eq!(BowAttackGoal::divergence(Difficulty::Hard), 2.0);
    }
}
