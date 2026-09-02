use pumpkin_protocol::java::client::play::CWorldEvent;
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
        48.0
    }
}

impl Goal for BlazeShootFireballGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(blaze) = self.blaze.upgrade() else {
            return false;
        };
        let target = blaze.entity.get_target();
        target.is_some_and(|t| t.get_entity().is_alive())
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        let Some(blaze) = self.blaze.upgrade() else {
            return false;
        };
        let target = blaze.entity.get_target();
        target.is_some_and(|t| t.get_entity().is_alive())
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

    fn tick(&mut self, _mob: &dyn Mob) {
        self.attack_time -= 1;

        let Some(blaze) = self.blaze.upgrade() else {
            return;
        };

        let target = blaze.entity.get_target();
        let Some(target) = target else {
            return;
        };

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
                blaze.entity.try_attack(&*blaze, &*target);
            }
        } else if distance_sq < Self::get_follow_distance().powi(2) && has_line_of_sight {
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
                    let chunk_pos = blaze.entity.living_entity.entity.chunk_pos.load();
                    blaze
                        .entity
                        .living_entity
                        .entity
                        .world
                        .load()
                        .broadcast_to_chunk(
                            chunk_pos,
                            &CWorldEvent::new(
                                1018,
                                blaze.entity.living_entity.entity.block_pos.load(),
                                0,
                                false,
                            ),
                        );

                    let world = blaze.entity.living_entity.entity.world.load_full();
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
                    world.spawn_entity(Arc::new(fireball));
                }
            }

            blaze
                .entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity(&*blaze, &target);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
