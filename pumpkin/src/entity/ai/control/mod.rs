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

    /// Strafe relative to the mob's facing direction. `forwards` and `right` are
    /// roughly in `[-1.0, 1.0]`. Move controls that don't support strafing keep
    /// the default no-op.
    fn strafe(&mut self, _forwards: f32, _right: f32) {}
}
