use crate::entity::{EntityBase, mob::Mob};
use pumpkin_util::math::subtract_angles;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

pub mod body_rotation_control;
pub mod fish_move_control;
pub mod flying_move_control;
pub mod jump_control;
pub mod look_control;
pub mod move_control;
pub mod smooth_swimming_look_control;
pub mod smooth_swimming_move_control;

pub trait Control: Send + Sync {
    fn change_angle(&self, start: f32, end: f32, max_change: f32) -> f32 {
        let i = subtract_angles(start, end);
        let j = i.clamp(-max_change, max_change);
        start + j
    }
}

pub trait MoveControlTrait: Control {
    fn tick(&mut self, mob: &dyn Mob);

    fn set_wanted_position(&mut self, _x: f64, _y: f64, _z: f64, _speed_modifier: f64) {}

    fn strafe(&mut self, _forward: f32, _right: f32) {}

    fn has_wanted(&self) -> bool {
        false
    }
}

pub trait LookControlTrait: Control {
    fn tick(&mut self, mob: &dyn Mob);

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    );

    fn look_at(&mut self, mob: &dyn Mob, x: f64, y: f64, z: f64) {
        self.look_at_with_range(
            x,
            y,
            z,
            mob.get_max_look_yaw_change(),
            mob.get_max_look_pitch_change(),
        );
    }

    fn look_at_position(&mut self, mob: &dyn Mob, position: Vector3<f64>) {
        self.look_at(mob, position.x, position.y, position.z);
    }

    fn look_at_entity(&mut self, mob: &dyn Mob, entity: &Arc<dyn EntityBase>) {
        self.look_at_position(mob, entity.get_entity().get_eye_pos());
    }

    fn look_at_entity_with_range(
        &mut self,
        entity: &Arc<dyn EntityBase>,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        let position = entity.get_entity().get_eye_pos();
        self.look_at_with_range(
            position.x,
            position.y,
            position.z,
            max_yaw_change,
            max_pitch_change,
        );
    }
}
