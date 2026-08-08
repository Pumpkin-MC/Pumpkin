use pumpkin_data::attributes::Attributes;

use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;

/// Vanilla `FlyingMoveControl`.
///
/// This is deliberately separate from the ground `MoveControl`: vanilla selects the
/// `FLYING_SPEED` attribute while airborne, writes vertical movement input from the wanted
/// position, and toggles no-gravity for every movement operation. Those details are observable
/// in entity movement and metadata, so a flying mob must not be driven by the ground controller.
pub struct FlyingMoveControl {
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    speed_modifier: f64,
    max_turn: f32,
    hovers_in_place: bool,
    operation: Operation,
}

impl FlyingMoveControl {
    #[must_use]
    pub const fn new(max_turn: f32, hovers_in_place: bool) -> Self {
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

impl Default for FlyingMoveControl {
    fn default() -> Self {
        Self::new(10.0, false)
    }
}

impl Control for FlyingMoveControl {}

impl MoveControlTrait for FlyingMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let living_entity = &mob.get_mob_entity().living_entity;
        let entity = &living_entity.entity;

        if self.operation == Operation::MoveTo {
            self.operation = Operation::Wait;
            if !entity.has_no_gravity() {
                entity.set_has_no_gravity(true);
            }

            let pos = entity.pos.load();
            let xd = self.wanted_x - pos.x;
            let yd = self.wanted_y - pos.y;
            let zd = self.wanted_z - pos.z;
            let dd = xd.mul_add(xd, yd.mul_add(yd, zd * zd));
            if dd < 2.5000003e-7 {
                set_vertical_and_forward_input(living_entity, 0.0, 0.0);
                return;
            }

            let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
            entity
                .yaw
                .store(self.change_angle(entity.yaw.load(), y_rot_d, 90.0));

            let speed_attribute = if entity.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                &Attributes::MOVEMENT_SPEED
            } else {
                &Attributes::FLYING_SPEED
            };
            let speed = self.speed_modifier * living_entity.get_attribute_value(speed_attribute);
            living_entity.set_speed(speed);

            let horizontal_distance = xd.hypot(zd);
            if yd.abs() > 1.0e-5 || horizontal_distance.abs() > 1.0e-5 {
                let x_rot_d = -(yd.atan2(horizontal_distance).to_degrees() as f32);
                entity.set_pitch(self.change_angle(entity.pitch.load(), x_rot_d, self.max_turn));
                set_vertical_input(living_entity, if yd > 0.0 { speed } else { -speed });
            }
        } else {
            if !self.hovers_in_place && entity.has_no_gravity() {
                entity.set_has_no_gravity(false);
            }
            set_vertical_and_forward_input(living_entity, 0.0, 0.0);
        }
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        self.operation = Operation::MoveTo;
    }

    fn get_wanted_x(&self) -> f64 {
        self.wanted_x
    }

    fn get_wanted_y(&self) -> f64 {
        self.wanted_y
    }

    fn get_wanted_z(&self) -> f64 {
        self.wanted_z
    }
}

fn set_vertical_input(living_entity: &crate::entity::living::LivingEntity, y: f64) {
    let mut input = living_entity.movement_input.load();
    input.y = y;
    living_entity.movement_input.store(input);
}

fn set_vertical_and_forward_input(
    living_entity: &crate::entity::living::LivingEntity,
    y: f64,
    z: f64,
) {
    let mut input = living_entity.movement_input.load();
    input.y = y;
    input.z = z;
    living_entity.movement_input.store(input);
}
