use std::sync::Arc;

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{
    Entity, EntityBase,
    mob::Mob,
    projectile::{arrow::ArrowPickup, trident::TridentEntity},
};

/// `Drowned.DrownedTridentAttackGoal` (`Drowned.java:531-557`), built on the same
/// `RangedAttackGoal` timing/range pattern skeletons use for bows.
///
/// Mirrors `ranged_bow_attack.rs::RangedBowAttackGoal`. Registered alongside
/// `DrownedAttackGoal` at priority 2 (`Drowned.java:93-94`); vanilla's `canUse` gate
/// (`getMainHandItem().is(TRIDENT)`) is what actually decides which of the two runs, since the
/// goal selector only allows one `Controls::MOVE`-flagged goal to run at a time.
pub struct DrownedTridentAttackGoal {
    attack_interval: i32,
    attack_time: i32,
    see_time: i32,
    speed: f64,
    range: f64,
}

impl DrownedTridentAttackGoal {
    #[must_use]
    pub fn new(attack_interval: i32, range: f64) -> Box<Self> {
        Box::new(Self {
            attack_interval,
            attack_time: -1,
            see_time: 0,
            speed: 1.0,
            range,
        })
    }

    async fn holding_trident(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let held_item = mob.get_mob_entity().living_entity.held_item(entity).await;
        held_item.lock().await.item == &Item::TRIDENT
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        let entity = mob.get_entity();
        let target_entity = target.get_entity();
        let from = entity.get_eye_pos();
        let to = target_entity.get_eye_pos();
        if from.squared_distance_to_vec(&to) > 128.0 * 128.0 {
            return false;
        }

        let world = entity.world.load_full();
        if !Arc::ptr_eq(&world, &target_entity.world.load_full()) {
            return false;
        }

        world
            .raycast_collision(from, to, async |block_pos, world| {
                !world.get_block_state(block_pos).collision_shapes.is_empty()
            })
            .await
            .is_none()
    }

    /// `Drowned#performRangedAttack`: `xd = target.getX() - this.getX()`,
    /// `zd = target.getZ() - this.getZ()` (from the shooter's own position, not the
    /// trident's), `yd = target.getY(1/3) - trident.getY()`, then
    /// `spawnProjectileUsingShoot(..., yd + distanceToTarget * 0.2F, ...)` where
    /// `distanceToTarget = sqrt(xd*xd + zd*zd)`.
    fn target_vector_from_positions(
        shooter_pos: Vector3<f64>,
        trident_y: f64,
        target_pos: Vector3<f64>,
        target_eye_height: f64,
    ) -> Vector3<f64> {
        let xd = target_pos.x - shooter_pos.x;
        let zd = target_pos.z - shooter_pos.z;
        let yd = target_pos.y + target_eye_height / 3.0 - trident_y;
        let horizontal_distance = xd.hypot(zd);
        Vector3::new(xd, yd + horizontal_distance * 0.2, zd)
    }

    fn target_vector(shooter: &Entity, trident_y: f64, target: &dyn EntityBase) -> Vector3<f64> {
        Self::target_vector_from_positions(
            shooter.pos.load(),
            trident_y,
            target.get_entity().pos.load(),
            target.get_entity().get_eye_height(),
        )
    }

    async fn shoot(&self, mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let held_item = mob.get_mob_entity().living_entity.held_item(shooter).await;
        let trident_item = {
            let held = held_item.lock().await;
            if held.item == &Item::TRIDENT {
                held.clone()
            } else {
                ItemStack::new(1, &Item::TRIDENT)
            }
        };

        let trident_entity = Entity::new(
            world.clone(),
            shooter.pos.load(),
            &pumpkin_data::entity::EntityType::TRIDENT,
        );
        let trident = TridentEntity::new_shot(
            trident_entity,
            shooter,
            trident_item,
            ArrowPickup::Disallowed,
        );
        let trident_y = trident.get_entity().pos.load().y;
        let direction = Self::target_vector(shooter, trident_y, target);
        // `spawnProjectileUsingShoot(..., 1.6F, 14 - level.getDifficulty().getId() * 4)`.
        // Normal difficulty is the server default (matching the same documented limitation in
        // `RangedBowAttackGoal::shoot`), giving a 10 degree inaccuracy.
        trident.set_velocity(direction.x, direction.y, direction.z, 1.6, 10.0);
        world.spawn_entity(Arc::new(trident)).await;

        let sound = CSoundEffect::new(
            IdOr::Id(Sound::EntityDrownedShoot as u16),
            SoundCategory::Hostile,
            &shooter.pos.load(),
            1.0,
            1.0 / (rand::random::<f32>() * 0.4 + 0.8),
            0.0,
        );
        world.broadcast_to_chunk(shooter.chunk_pos.load(), &sound);
    }
}

impl Goal for DrownedTridentAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let has_target = mob
                .get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive());
            has_target && Self::holding_trident(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let has_target = mob
                .get_mob_entity()
                .target
                .lock()
                .await
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive());
            let holding_trident = Self::holding_trident(mob).await;
            let navigation_active = !mob.get_mob_entity().navigator.lock().unwrap().is_idle();
            has_target && (holding_trident || navigation_active)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().set_attacking(false);
            self.attack_time = -1;
            self.see_time = 0;
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
            let has_line_of_sight = Self::has_line_of_sight(mob, target.as_ref()).await;
            if has_line_of_sight {
                self.see_time += 1;
            } else {
                self.see_time = 0;
            }

            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            if distance_squared > self.range * self.range || self.see_time < 5 {
                mob.get_mob_entity().navigator.lock().unwrap().set_progress(
                    crate::entity::ai::pathfinder::NavigatorGoal {
                        current_progress: shooter.pos.load(),
                        destination: target_pos,
                        speed: self.speed,
                    },
                );
            } else {
                mob.get_mob_entity().navigator.lock().unwrap().stop();
            }

            self.attack_time -= 1;
            if self.attack_time == 0 {
                if !has_line_of_sight {
                    return;
                }
                self.shoot(mob, target.as_ref()).await;
                self.attack_time = self.attack_interval;
            } else if self.attack_time < 0 {
                self.attack_time = self.attack_interval;
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
    use super::DrownedTridentAttackGoal;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn preserves_vanilla_trident_interval_and_radius() {
        let goal = DrownedTridentAttackGoal::new(40, 10.0);
        assert_eq!(goal.attack_interval, 40);
        assert_eq!(goal.range, 10.0);
    }

    #[test]
    fn adds_vanilla_ballistic_vertical_lead() {
        let direction = DrownedTridentAttackGoal::target_vector_from_positions(
            Vector3::new(0.0, 1.42, 0.0),
            1.52,
            Vector3::new(3.0, 0.0, 4.0),
            1.8,
        );

        assert_eq!(direction.x, 3.0);
        assert_eq!(direction.z, 4.0);
        assert!((direction.y - 0.08).abs() < f64::EPSILON);
    }
}
