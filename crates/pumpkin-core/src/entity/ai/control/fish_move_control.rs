use pumpkin_data::attributes::Attributes;

use super::{
    Control, MoveControlTrait,
    move_control::{MoveControl, Operation},
};
use crate::entity::mob::Mob;

const BUOYANCY: f64 = 0.005;
const SPEED_LERP: f32 = 0.125;
const VERTICAL_ACCELERATION: f64 = 0.1;

#[derive(Default)]
pub struct FishMoveControl {
    inner: MoveControl,
}

impl Control for FishMoveControl {}

impl MoveControlTrait for FishMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        if entity.is_submerged_in_water() {
            entity
                .velocity
                .store(entity.velocity.load().add_raw(0.0, BUOYANCY, 0.0));
        }
        if self.inner.operation == Operation::MoveTo && !mob_entity.steered().navigator().is_done()
        {
            let target_speed = (self.inner.speed_modifier
                * living.get_attribute_value(&Attributes::MOVEMENT_SPEED))
                as f32;
            mob_entity
                .set_speed(living.get_speed() + SPEED_LERP * (target_speed - living.get_speed()));
            let pos = entity.pos.load();
            let xd = self.inner.wanted_x - pos.x;
            let yd = self.inner.wanted_y - pos.y;
            let zd = self.inner.wanted_z - pos.z;
            if yd != 0.0 {
                let distance = (xd * xd + yd * yd + zd * zd).sqrt();
                entity.velocity.store(entity.velocity.load().add_raw(
                    0.0,
                    f64::from(living.get_speed()) * (yd / distance) * VERTICAL_ACCELERATION,
                    0.0,
                ));
            }
            if xd != 0.0 || zd != 0.0 {
                let yaw = MoveControl::rotlerp(
                    entity.yaw.load(),
                    zd.atan2(xd).to_degrees() as f32 - 90.0,
                    MoveControl::MAX_TURN,
                );
                entity.yaw.store(yaw);
                entity.body_yaw.store(yaw);
            }
        } else {
            mob_entity.set_speed(0.0);
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.inner.set_wanted_position(x, y, z, speed_modifier);
    }

    fn strafe(&mut self, forwards: f32, right: f32) {
        self.inner.strafe(forwards, right);
    }

    fn has_wanted(&self) -> bool {
        self.inner.has_wanted()
    }
}
