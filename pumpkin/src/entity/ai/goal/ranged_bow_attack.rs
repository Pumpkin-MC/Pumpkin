use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Controls, Goal, GoalFuture},
    mob::Mob,
    projectile::arrow::{ArrowEntity, ArrowPickup},
};

/// The common bow attack loop used by skeleton-family mobs.
///
/// This follows vanilla's `RangedBowAttackGoal` timing while Pumpkin does not
/// yet model item-use state.  A shot is still created by the same projectile
/// path as player bows, with the vanilla skeleton trajectory adjustment from
/// `AbstractSkeleton#performRangedAttack`.
pub struct RangedBowAttackGoal {
    attack_interval: i32,
    attack_cooldown: i32,
    range: f64,
}

impl RangedBowAttackGoal {
    #[must_use]
    pub const fn new(attack_interval: i32, range: f64) -> Self {
        Self {
            attack_interval,
            attack_cooldown: 0,
            range,
        }
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let entity = mob.get_entity();
        entity
            .world
            .load_full()
            .raycast(
                entity.get_eye_pos(),
                target.get_entity().get_eye_pos(),
                async |block_pos, world| world.get_block_state(block_pos).is_solid(),
            )
            .await
            .is_none()
    }

    fn target_vector(shooter: &Entity, target: &dyn EntityBase) -> Vector3<f64> {
        // `ArrowEntity::new_shot` starts an arrow at eye height minus 0.1.
        let mut arrow_pos = shooter.get_eye_pos();
        arrow_pos.y -= 0.1;
        let target_pos = target.get_entity().pos.load();
        Self::target_vector_from_positions(
            arrow_pos,
            target_pos,
            target.get_entity().get_eye_height(),
        )
    }

    fn target_vector_from_positions(
        arrow_pos: Vector3<f64>,
        target_pos: Vector3<f64>,
        target_eye_height: f64,
    ) -> Vector3<f64> {
        let horizontal_distance = (target_pos.x - arrow_pos.x).hypot(target_pos.z - arrow_pos.z);

        // Vanilla: target.getY(1 / 3) - arrow.getY() + horizontalDistance * 0.2.
        Vector3::new(
            target_pos.x - arrow_pos.x,
            target_pos.y + target_eye_height / 3.0 - arrow_pos.y + horizontal_distance * 0.2,
            target_pos.z - arrow_pos.z,
        )
    }

    async fn shoot(&self, mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let arrow_entity = Entity::new(world.clone(), shooter.pos.load(), &EntityType::ARROW);
        let arrow = ArrowEntity::new_shot(arrow_entity, shooter, ArrowPickup::Disallowed);
        let direction = Self::target_vector(shooter, target);

        // `AbstractSkeleton#performRangedAttack`: power 1.6, inaccuracy 14 - difficulty * 4.
        // Normal difficulty is the server default, so use its 10 degree inaccuracy until
        // local difficulty is represented by the world API.
        arrow.set_velocity(direction.x, direction.y, direction.z, 1.6, 10.0);
        world.spawn_entity(Arc::new(arrow)).await;

        let sound = CSoundEffect::new(
            IdOr::Id(Sound::EntitySkeletonShoot as u16),
            SoundCategory::Hostile,
            &shooter.pos.load(),
            1.0,
            1.0 / (rand::random::<f32>() * 0.4 + 0.8),
            0.0,
        );
        world.broadcast_to_chunk(shooter.chunk_pos.load(), &sound);
    }
}

impl Goal for RangedBowAttackGoal {
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
            self.attack_cooldown = 0;
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
            let target_pos = target.get_entity().pos.load();
            let distance_squared = shooter.pos.load().squared_distance_to_vec(&target_pos);

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            self.attack_cooldown = (self.attack_cooldown - 1).max(0);

            if distance_squared > self.range * self.range {
                mob.get_mob_entity().navigator.lock().unwrap().set_progress(
                    crate::entity::ai::pathfinder::NavigatorGoal {
                        current_progress: shooter.pos.load(),
                        destination: target_pos,
                        speed: 1.0,
                    },
                );
                return;
            }

            mob.get_mob_entity().navigator.lock().unwrap().stop();
            if self.attack_cooldown == 0 && Self::has_line_of_sight(mob, target.as_ref()).await {
                self.shoot(mob, target.as_ref()).await;
                self.attack_cooldown = self.attack_interval;
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
    use super::RangedBowAttackGoal;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn preserves_vanilla_skeleton_bow_interval() {
        let goal = RangedBowAttackGoal::new(20, 15.0);
        assert_eq!(goal.attack_interval, 20);
        assert_eq!(goal.range, 15.0);
    }

    #[test]
    fn adds_vanilla_ballistic_vertical_lead() {
        let direction = RangedBowAttackGoal::target_vector_from_positions(
            Vector3::new(0.0, 1.52, 0.0),
            Vector3::new(3.0, 0.0, 4.0),
            1.8,
        );

        assert_eq!(direction.x, 3.0);
        assert_eq!(direction.z, 4.0);
        // target Y + targetEyeHeight / 3 - arrow Y + horizontalDistance * 0.2
        assert!((direction.y - 0.08).abs() < f64::EPSILON);
    }
}
