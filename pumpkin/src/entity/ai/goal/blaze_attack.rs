use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;

use crate::entity::{
    Entity,
    ai::goal::{Controls, Goal, GoalFuture},
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
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(blaze) = self.blaze.upgrade() else {
                return false;
            };
            let target = blaze.entity.target.lock().await.clone();
            if target.is_some() {
                // TODO: check is_alive
                true
            } else {
                false
            }
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(blaze) = self.blaze.upgrade() else {
                return false;
            };
            let target = blaze.entity.target.lock().await.clone();
            if target.is_some() {
                // TODO: check is_alive
                true
            } else {
                false
            }
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_step = 0;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(blaze) = self.blaze.upgrade() {
                blaze.set_charged(false);
            }
            self.last_seen = 0;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_time -= 1;

            let Some(blaze) = self.blaze.upgrade() else {
                return;
            };

            let target = blaze.entity.target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            // TODO: hasLineOfSight check
            let has_line_of_sight = true;

            if has_line_of_sight {
                self.last_seen = 0;
            } else {
                self.last_seen += 1;
            }

            let blaze_pos = blaze.entity.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();

            let dx = target_pos.x - blaze_pos.x;
            let dy = target_pos.y - blaze_pos.y;
            let dz = target_pos.z - blaze_pos.z;

            let distance_sq = dx * dx + dy * dy + dz * dz;

            if distance_sq < 4.0 {
                if !has_line_of_sight {
                    return;
                }

                if self.attack_time <= 0 {
                    self.attack_time = 20;
                    // TODO: doHurtTarget
                }

                // TODO: set wanted position to target
            } else if distance_sq < Self::get_follow_distance().powi(2) && has_line_of_sight {
                let target_y_offset = target_pos.y + 0.5; // roughly target.getY(0.5)
                let blaze_y_offset = blaze_pos.y + 0.5; // roughly blaze.getY(0.5)
                let yd = target_y_offset - blaze_y_offset;

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
                        let distance = distance_sq.sqrt();
                        let sqd = distance.sqrt() * 0.5;

                        // play shoot sound
                        blaze
                            .entity
                            .living_entity
                            .entity
                            .world
                            .load()
                            .broadcast_packet_all(&CWorldEvent::new(
                                1018,
                                blaze.entity.living_entity.entity.block_pos.load(),
                                0,
                                false,
                            ))
                            .await;

                        for _ in 0..1 {
                            // Vanilla loops 1 time
                            // Calculate spread
                            let direction = {
                                let mut rng = rand::rng();
                                let dir_x = (dx - 2.297 * sqd)
                                    + rng.random_range(0.0..1.0) * (2.297 * sqd * 2.0);
                                let dir_z = (dz - 2.297 * sqd)
                                    + rng.random_range(0.0..1.0) * (2.297 * sqd * 2.0);
                                Vector3::new(dir_x, yd, dir_z).normalize()
                            };

                            // Spawn SmallFireball
                            let world = blaze.entity.living_entity.entity.world.load();
                            let uuid = uuid::Uuid::new_v4();

                            let mut pos = blaze.entity.living_entity.entity.pos.load();
                            pos.y += blaze.entity.living_entity.entity.get_eye_height() - 0.1;

                            let base_entity = Entity::from_uuid(
                                uuid,
                                world.clone(),
                                pos,
                                &pumpkin_data::entity::EntityType::SMALL_FIREBALL,
                            );

                            let fireball = SmallFireballEntity::new_shot(
                                base_entity,
                                &blaze.entity.living_entity.entity,
                            );
                            fireball.thrown.entity.velocity.store(direction);

                            world.spawn_entity(Arc::new(fireball)).await;
                        }
                    }
                }

                // Look at target
                blaze
                    .entity
                    .look_control
                    .lock()
                    .await
                    .look_at_entity(&*blaze, &target);
            } else if self.last_seen < 5 {
                // TODO: set wanted position to target
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
