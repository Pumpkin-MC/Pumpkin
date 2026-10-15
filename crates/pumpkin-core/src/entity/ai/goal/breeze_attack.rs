use std::sync::{Arc, Weak, atomic::AtomicBool};

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Controls, Goal},
    mob::{Mob, breeze::BreezeEntity},
    projectile::{
        ThrownItemEntity,
        wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity},
    },
};

pub struct BreezeAttackGoal {
    breeze: Weak<BreezeEntity>,
    shoot_cooldown: i32,
    jump_cooldown: i32,
    charging_shoot: i32,
}

impl BreezeAttackGoal {
    #[must_use]
    pub fn new(breeze: Weak<BreezeEntity>) -> Box<Self> {
        Box::new(Self {
            breeze,
            shoot_cooldown: 40,
            jump_cooldown: 20,
            charging_shoot: 0,
        })
    }
}

impl Goal for BreezeAttackGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(breeze) = self.breeze.upgrade() else {
            return false;
        };
        let Some(target) = breeze.mob_entity.get_target() else {
            return false;
        };
        target.get_entity().is_alive() && breeze.can_attack(target.as_ref())
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(breeze) = self.breeze.upgrade() else {
            return false;
        };
        let Some(target) = breeze.mob_entity.get_target() else {
            return false;
        };
        target.get_entity().is_alive()
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.charging_shoot = 0;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(breeze) = self.breeze.upgrade() else {
            return;
        };
        let Some(target) = breeze.mob_entity.get_target() else {
            return;
        };

        if self.shoot_cooldown > 0 {
            self.shoot_cooldown -= 1;
        }
        if self.jump_cooldown > 0 {
            self.jump_cooldown -= 1;
        }

        let breeze_entity = breeze.get_entity();
        let target_entity = target.get_entity();

        let breeze_pos = breeze_entity.pos.load();
        let target_pos = target_entity.pos.load();
        let dist_sq = breeze_pos.squared_distance_to_vec(&target_pos);
        let world = breeze_entity.world.load();

        // Always face target when engaged
        breeze
            .mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 10.0, 10.0);

        // 1. If target is too close (< 4 blocks, dist_sq < 16.0), leap away!
        if dist_sq < 16.0 && self.jump_cooldown <= 0 {
            self.jump_cooldown = rand::random_range(40..=60);
            self.charging_shoot = 0;

            let mut diff = breeze_pos - target_pos;
            diff.y = 0.0;
            let len = (diff.x * diff.x + diff.z * diff.z).sqrt();
            let (dx, dz) = if len > 0.001 {
                (diff.x / len * 0.8, diff.z / len * 0.8)
            } else {
                (0.5, 0.5)
            };

            world.play_sound_fine(
                Sound::EntityBreezeJump,
                SoundCategory::Hostile,
                &breeze_pos,
                1.0,
                1.0,
            );

            breeze_entity.set_velocity(Vector3::new(dx, 0.75, dz));
            return;
        }

        // 2. If within combat range (4 to 24 blocks, dist_sq <= 576.0)
        if dist_sq <= 576.0 && mob.has_line_of_sight(target_entity) {
            if self.charging_shoot > 0 {
                self.charging_shoot -= 1;
                if self.charging_shoot == 0 {
                    // Spawn breeze wind charge projectile
                    let firing_y = breeze_pos.y + f64::from(breeze_entity.height()) * 0.5 + 0.3;
                    let spawn_pos = Vector3::new(breeze_pos.x, firing_y, breeze_pos.z);

                    let target_y = target_pos.y + 0.3;
                    let diff = Vector3::new(
                        target_pos.x - breeze_pos.x,
                        target_y - firing_y,
                        target_pos.z - breeze_pos.z,
                    );
                    let len = (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt();
                    let vel = if len > 0.001 {
                        Vector3::new(
                            diff.x / len * 0.7,
                            diff.y / len * 0.7,
                            diff.z / len * 0.7,
                        )
                    } else {
                        Vector3::new(0.0, 0.0, 0.7)
                    };

                    let proj_entity = Entity::from_uuid(
                        uuid::Uuid::new_v4(),
                        world.clone(),
                        spawn_pos,
                        &EntityType::BREEZE_WIND_CHARGE,
                    );
                    proj_entity.set_velocity(vel);

                    let thrown = ThrownItemEntity {
                        entity: proj_entity,
                        owner_id: Some(breeze_entity.entity_id),
                        collides_with_projectiles: false,
                        has_hit: AtomicBool::new(false),
                        gravity: WIND_CHARGE_GRAVITY,
                    };
                    let wind_charge = WindChargeEntity::new_breeze(thrown);
                    world.spawn_entity(Arc::new(wind_charge));

                    world.play_sound_fine(
                        Sound::EntityBreezeShoot,
                        SoundCategory::Hostile,
                        &breeze_pos,
                        1.5,
                        1.0,
                    );

                    self.shoot_cooldown = rand::random_range(40..=70);
                }
            } else if self.shoot_cooldown <= 0 {
                self.charging_shoot = 15; // 15 ticks delay matching vanilla SHOOT_INITIAL_DELAY_TICKS
                world.play_sound_fine(
                    Sound::EntityBreezeInhale,
                    SoundCategory::Hostile,
                    &breeze_pos,
                    1.0,
                    1.0,
                );
            } else if self.jump_cooldown <= 0 && rand::random_range(0..30) == 0 {
                // Periodic tactical leap/slide around target
                self.jump_cooldown = rand::random_range(50..=80);
                world.play_sound_fine(
                    Sound::EntityBreezeSlide,
                    SoundCategory::Hostile,
                    &breeze_pos,
                    1.0,
                    1.0,
                );
                let angle = rand::random_range(0.0..(std::f64::consts::PI * 2.0));
                let vx = angle.cos() * 0.6;
                let vz = angle.sin() * 0.6;
                breeze_entity.set_velocity(Vector3::new(vx, 0.5, vz));
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
