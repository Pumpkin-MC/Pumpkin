use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct FlyingMoveControl {
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
    pub speed_modifier: f64,
    pub max_turn: i32,
    pub hovers_in_place: bool,
    pub operation: Operation,
}

impl FlyingMoveControl {
    #[must_use]
    pub const fn new(max_turn: i32, hovers_in_place: bool) -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier: 0.0,
            max_turn,
            hovers_in_place,
            operation: Operation::Wait,
        }
    }
}

impl Control for FlyingMoveControl {}

impl MoveControlTrait for FlyingMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;

        if self.operation == Operation::MoveTo {
            self.operation = Operation::Wait;
            entity.set_has_no_gravity(true);

            let pos = entity.pos.load();
            let xd = self.wanted_x - pos.x;
            let yd = self.wanted_y - pos.y;
            let zd = self.wanted_z - pos.z;
            let dd = xd * xd + yd * yd + zd * zd;

            if dd < 2.5000003E-7 {
                living_entity
                    .movement_input
                    .store(Vector3::new(0.0, 0.0, 0.0));
                return;
            }

            let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
            entity
                .yaw
                .store(self.change_angle(entity.yaw.load(), y_rot_d, 90.0));

            let speed = if entity.on_ground.load(Ordering::Relaxed) {
                let movement_speed = living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
                (self.speed_modifier * movement_speed) as f32
            } else {
                let flying_speed = living_entity.get_attribute_value(&Attributes::FLYING_SPEED);
                (self.speed_modifier * flying_speed) as f32
            };

            let sd = xd.hypot(zd);
            if yd.abs() > 1.0E-5 || sd > 1.0E-5 {
                let x_rot_d = -((yd.atan2(sd).to_degrees()) as f32);
                entity.pitch.store(self.change_angle(
                    entity.pitch.load(),
                    x_rot_d,
                    self.max_turn as f32,
                ));
                let yya = if yd > 0.0 {
                    speed as f64
                } else {
                    -(speed as f64)
                };
                living_entity
                    .movement_input
                    .store(Vector3::new(0.0, yya, speed as f64));
            } else {
                living_entity
                    .movement_input
                    .store(Vector3::new(0.0, 0.0, speed as f64));
            }
        } else {
            if !self.hovers_in_place {
                entity.set_has_no_gravity(false);
            }
            living_entity
                .movement_input
                .store(Vector3::new(0.0, 0.0, 0.0));
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        self.operation = Operation::MoveTo;
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }
}
