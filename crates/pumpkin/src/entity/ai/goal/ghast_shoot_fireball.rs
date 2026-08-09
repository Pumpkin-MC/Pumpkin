use std::sync::Arc;
use std::sync::Weak;

use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Controls, Goal, GoalFuture},
    mob::Mob,
    mob::ghast::GhastEntity,
    projectile::fireball::FireballEntity,
};

const FOLLOW_DISTANCE_SQ: f64 = 4096.0;
const CHARGE_SOUND_TICK: i32 = 10;
const SHOOT_TICK: i32 = 20;
const COOLDOWN_AFTER_SHOT: i32 = -40;

/// Vanilla: `Ghast.GhastShootFireballGoal` (Ghast.java:325-388, priority 7 at Ghast.java:59).
pub struct GhastShootFireballGoal {
    ghast: Weak<GhastEntity>,
    charge_time: i32,
}

impl GhastShootFireballGoal {
    #[must_use]
    pub const fn new(ghast: Weak<GhastEntity>) -> Self {
        Self {
            ghast,
            charge_time: 0,
        }
    }

    /// Vanilla: `target.distanceToSqr(this.ghast) < 4096.0 && this.ghast.hasLineOfSight(target)`
    /// (Ghast.java:358).
    const fn in_range(distance_sq: f64) -> bool {
        distance_sq < FOLLOW_DISTANCE_SQ
    }

    /// Vanilla: `this.ghast.setCharging(this.chargeTime > 10)` (Ghast.java:385).
    const fn is_charging(charge_time: i32) -> bool {
        charge_time > 10
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        mob.get_mob_entity().has_line_of_sight(target).await
    }
}

impl Goal for GhastShootFireballGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(ghast) = self.ghast.upgrade() else {
                return false;
            };
            ghast.mob_entity.target.lock().await.is_some()
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.charge_time = 0;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(ghast) = self.ghast.upgrade() {
                ghast.set_charging(false);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(ghast) = self.ghast.upgrade() else {
                return;
            };

            let target = ghast.mob_entity.target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            let ghast_pos = ghast.mob_entity.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let dx = target_pos.x - ghast_pos.x;
            let dy = target_pos.y - ghast_pos.y;
            let dz = target_pos.z - ghast_pos.z;
            let distance_sq = dx * dx + dy * dy + dz * dz;

            if Self::in_range(distance_sq)
                && Self::has_line_of_sight(&*ghast, target.as_ref()).await
            {
                let world = ghast.mob_entity.living_entity.entity.world.load();
                self.charge_time += 1;

                if self.charge_time == CHARGE_SOUND_TICK {
                    let chunk_pos = ghast.mob_entity.living_entity.entity.chunk_pos.load();
                    world.broadcast_to_chunk(
                        chunk_pos,
                        &CWorldEvent::new(
                            1015,
                            ghast.mob_entity.living_entity.entity.block_pos.load(),
                            0,
                            false,
                        ),
                    );
                }

                if self.charge_time == SHOOT_TICK {
                    let entity = &ghast.mob_entity.living_entity.entity;
                    let view_vector = Vector3::rotation_vector(
                        f64::from(entity.pitch.load()),
                        f64::from(entity.yaw.load()),
                    );

                    let entity_dimension = entity.entity_dimension.load();
                    let ghast_mid_y = ghast_pos.y + f64::from(entity_dimension.height) * 0.5;
                    let target_height = target.get_living_entity().map_or(0.0, |living| {
                        f64::from(living.entity.entity_dimension.load().height)
                    });
                    let target_mid_y = target_pos.y + target_height * 0.5;

                    let xdd = target_pos.x - (ghast_pos.x + view_vector.x * 4.0);
                    let ydd = target_mid_y - (0.5 + ghast_mid_y);
                    let zdd = target_pos.z - (ghast_pos.z + view_vector.z * 4.0);
                    let direction = Vector3::new(xdd, ydd, zdd).normalize();

                    let chunk_pos = entity.chunk_pos.load();
                    world.broadcast_to_chunk(
                        chunk_pos,
                        &CWorldEvent::new(1016, entity.block_pos.load(), 0, false),
                    );

                    let spawn_pos = Vector3::new(
                        ghast_pos.x + view_vector.x * 4.0,
                        ghast_mid_y + 0.5,
                        ghast_pos.z + view_vector.z * 4.0,
                    );

                    let base_entity = Entity::from_uuid(
                        Uuid::new_v4(),
                        world.clone(),
                        spawn_pos,
                        &pumpkin_data::entity::EntityType::FIREBALL,
                    );
                    let mut fireball = FireballEntity::new(base_entity);
                    fireball.explosion_power = f32::from(ghast.explosion_power());
                    fireball.thrown.owner_id = Some(entity.entity_id);
                    // Vanilla: `AbstractHurtingProjectile.assignDirectionalMovement` scales the
                    // normalized direction by `accelerationPower` (0.1) (AbstractHurtingProjectile.java:180-182).
                    fireball.thrown.entity.velocity.store(direction * 0.1);

                    world.spawn_entity(Arc::new(fireball)).await;
                    self.charge_time = COOLDOWN_AFTER_SHOT;
                }
            } else if self.charge_time > 0 {
                self.charge_time -= 1;
            }

            ghast.set_charging(Self::is_charging(self.charge_time));
        })
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::GhastShootFireballGoal;

    #[test]
    fn range_gate_matches_vanilla_sixty_four_blocks() {
        assert!(GhastShootFireballGoal::in_range(4095.99));
        assert!(!GhastShootFireballGoal::in_range(4096.0));
    }

    #[test]
    fn charging_flag_flips_above_ten_ticks() {
        assert!(!GhastShootFireballGoal::is_charging(10));
        assert!(GhastShootFireballGoal::is_charging(11));
        assert!(!GhastShootFireballGoal::is_charging(-40));
    }
}
