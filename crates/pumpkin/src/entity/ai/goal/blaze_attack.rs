use crate::entity::ai::util::RandomExt;
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::{
    Entity,
    ai::goal::{Controls, Goal},
    mob::Mob,
    mob::blaze::BlazeEntity,
    projectile::small_fireball::SmallFireballEntity,
};

pub struct BlazeShootFireballGoal {
    blaze: std::sync::Weak<BlazeEntity>,
    attack_step: i32,
    attack_time: i32,
    last_seen: i32,
}

impl BlazeShootFireballGoal {
    #[must_use]
    pub const fn new(blaze: std::sync::Weak<BlazeEntity>) -> Self {
        Self {
            blaze,
            attack_step: 0,
            attack_time: 0,
            last_seen: 0,
        }
    }

    const fn get_follow_distance() -> f64 {
        // TODO: use FOLLOW_RANGE
        48.0
    }
}

impl Goal for BlazeShootFireballGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(blaze) = self.blaze.upgrade() else {
            return false;
        };
        let Some(target) = blaze.entity.get_target() else {
            return false;
        };
        let Some(target_living) = target.get_living_entity() else {
            return false;
        };
        target.get_entity().is_alive() && blaze.entity.living_entity.can_attack(target_living)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.attack_step = 0;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(blaze) = self.blaze.upgrade() {
            blaze.set_charged(false);
        }
        self.last_seen = 0;
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.attack_time -= 1;

        let Some(blaze) = self.blaze.upgrade() else {
            return;
        };

        let target = blaze.entity.get_target();
        let Some(target) = target else {
            return;
        };

        let entity = &blaze.entity.living_entity.entity;
        let has_line_of_sight = mob.has_line_of_sight(target.get_entity());

        if has_line_of_sight {
            self.last_seen = 0;
        } else {
            self.last_seen += 1;
        }

        let blaze_pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();

        let dx = target_pos.x - blaze_pos.x;
        let dz = target_pos.z - blaze_pos.z;
        let distance_sq = blaze_pos.squared_distance_to_vec(&target_pos);

        if distance_sq < 4.0 {
            if !has_line_of_sight {
                return;
            }

            if self.attack_time <= 0 {
                self.attack_time = 20;
                blaze.entity.try_attack(&*blaze, target.as_ref());
            }

            blaze
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_wanted_position(target_pos.x, target_pos.y, target_pos.z, 1.0);
        } else if distance_sq < Self::get_follow_distance().powi(2) && has_line_of_sight {
            let yd = (target_pos.y + f64::from(target_entity.entity_dimension.load().height) * 0.5)
                - (blaze_pos.y + f64::from(entity.entity_dimension.load().height) * 0.5);

            if self.attack_time <= 0 {
                self.attack_step += 1;
                if self.attack_step == 1 {
                    self.attack_time = 60;
                    blaze.set_charged(true);
                } else if self.attack_step <= 4 {
                    self.attack_time = 6;
                } else {
                    self.attack_time = 100;
                    self.attack_step = 0;
                    blaze.set_charged(false);
                }

                if self.attack_step > 1 {
                    shoot_fireball(entity, dx, yd, dz, distance_sq);
                }
            }

            blaze
                .entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity_with_range(&target, 10.0, 10.0);
        } else if self.last_seen < 5 {
            blaze
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_wanted_position(target_pos.x, target_pos.y, target_pos.z, 1.0);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

/// One fireball, aimed with a triangular spread that widens with the distance.
fn shoot_fireball(blaze: &Entity, dx: f64, yd: f64, dz: f64, distance_sq: f64) {
    let spread = 2.297 * distance_sq.sqrt().sqrt() * 0.5;
    let world = blaze.world.load_full();

    world.broadcast_to_chunk(
        blaze.chunk_pos.load(),
        &CWorldEvent::new(1018, blaze.block_pos.load(), 0, false),
    );

    let direction = {
        let mut rng = rand::rng();
        Vector3::new(rng.triangle(dx, spread), yd, rng.triangle(dz, spread))
    };

    let blaze_pos = blaze.pos.load();
    let spawn_pos = Vector3::new(
        blaze_pos.x,
        blaze_pos.y + f64::from(blaze.entity_dimension.load().height) * 0.5 + 0.5,
        blaze_pos.z,
    );
    let base_entity = Entity::from_uuid(
        uuid::Uuid::new_v4(),
        world.clone(),
        spawn_pos,
        &pumpkin_data::entity::EntityType::SMALL_FIREBALL,
    );

    let fireball = SmallFireballEntity::new_shot(base_entity, blaze, direction);
    // `ThrownItemEntity::new` puts the projectile at the shooter's eye, re-anchor it.
    fireball.thrown.entity.pos.store(spawn_pos);
    world.spawn_entity(Arc::new(fireball));
}
