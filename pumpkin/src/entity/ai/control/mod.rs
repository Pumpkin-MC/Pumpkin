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

    /// Vanilla `MoveControl.strafe` — sideways/backwards input relative to the
    /// mob's facing, re-armed by the owning goal every tick.
    fn strafe(&mut self, _forwards: f32, _sideways: f32) {}

    /// True while a strafe request from this tick is pending, so the navigator's
    /// idle branch does not stomp it before the controller runs.
    fn is_strafing(&self) -> bool {
        false
    }
}
