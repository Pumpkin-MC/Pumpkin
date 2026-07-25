use crate::entity::mob::Mob;
use pumpkin_util::math::subtract_angles;

pub mod look_control;
pub mod move_control;

pub trait Control: Send + Sync {
    fn change_angle(&self, start: f32, end: f32, max_change: f32) -> f32 {
        let i = subtract_angles(start, end);
        let j = i.clamp(-max_change, max_change);
        start + j
    }
}

pub trait MoveControlTrait: Control {
    fn tick(&mut self, mob: &dyn Mob);

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64);

    fn stop(&mut self) {}
}
