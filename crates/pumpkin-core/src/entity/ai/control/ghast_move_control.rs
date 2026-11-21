use std::sync::atomic::Ordering;

use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct GhastMoveControl {
    pub wanted: Vector3<f64>,
    pub speed_modifier: f64,
    pub operation: Operation,
    pub float_duration: i32,
    pub careful: bool,
    pub is_still: Option<fn(&dyn Mob) -> bool>,
}

impl GhastMoveControl {
    #[must_use]
    pub fn new(careful: bool, is_still: Option<fn(&dyn Mob) -> bool>) -> Self {
        Self {
            wanted: Vector3::default(),
            speed_modifier: 1.0,
            operation: Operation::Wait,
            float_duration: 0,
            careful,
            is_still,
        }
    }

    fn can_reach(mob: &dyn Mob, step: Vector3<f64>, steps: i32) -> bool {
        let entity = mob.get_entity();
        let mut bb = entity.bounding_box.load();
        let world = entity.world.load();
        for _ in 1..steps {
            bb = bb.shift(step);
            if !world.no_collision(entity, bb) {
                return false;
            }
        }
        true
    }
}

impl Control for GhastMoveControl {}

impl MoveControlTrait for GhastMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if self.is_still.is_some_and(|check| check(mob)) {
            self.operation = Operation::Wait;
            let entity = mob.get_entity();
            let vel = entity.velocity.load();
            if vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0 {
                entity.velocity.store(Vector3::default());
                entity.velocity_dirty.store(true, Ordering::SeqCst);
            }
            mob.get_mob_entity().steered().navigator().stop();
            return;
        }

        if self.operation == Operation::MoveTo {
            self.float_duration -= 1;
            if self.float_duration <= 0 {
                self.float_duration += mob.get_random().random_range(2..=6);
                let pos = mob.get_entity().pos.load();
                let travel = Vector3::new(
                    self.wanted.x - pos.x,
                    self.wanted.y - pos.y,
                    self.wanted.z - pos.z,
                );
                let dist = travel.length();
                if dist > 0.0001 {
                    let step = travel / dist;
                    if Self::can_reach(mob, step, dist.ceil() as i32) {
                        let entity = mob.get_entity();
                        let current_vel = entity.velocity.load();
                        let flying_speed = mob.get_mob_entity().living_entity.get_attribute_value(
                            &pumpkin_data::attributes::Attributes::FLYING_SPEED,
                        ) * 5.0
                            / 3.0;
                        let speed = if flying_speed > 0.0 {
                            flying_speed
                        } else {
                            0.1
                        };
                        entity.velocity.store(current_vel + step * speed);
                    } else {
                        self.operation = Operation::Wait;
                    }
                }
            }
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted = Vector3::new(x, y, z);
        self.speed_modifier = speed_modifier;
        self.operation = Operation::MoveTo;
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        self.wanted
    }

    fn set_wait(&mut self) {
        self.operation = Operation::Wait;
    }
}
