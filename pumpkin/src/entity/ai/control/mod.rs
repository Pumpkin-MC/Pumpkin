use crate::entity::mob::Mob;
use pumpkin_util::math::subtract_angles;

pub mod ghast_move_control;
pub mod look_control;
pub mod move_control;
pub mod phantom_move_control;
pub mod vex_move_control;

pub trait Control: Send + Sync {
    fn change_angle(&self, start: f32, end: f32, max_change: f32) -> f32 {
        let i = subtract_angles(start, end);
        let j = i.clamp(-max_change, max_change);
        start + j
    }
}

pub trait MoveControlTrait: Control {
    fn tick(&mut self, mob: &dyn Mob);

    /// Vanilla `MoveControl.hasWanted`: whether a movement destination is currently set.
    /// Defaults to `false` for move controls that don't model a "wanted position" (e.g. the
    /// slime/sulfur-cube hop controllers).
    fn has_wanted(&self) -> bool {
        false
    }

    /// Vanilla `MoveControl.setWantedPosition`. Defaults to a no-op; only meaningful for move
    /// controls that implement `has_wanted`.
    fn set_wanted_position(&mut self, _x: f64, _y: f64, _z: f64, _speed_modifier: f64) {}

    /// Vanilla `MoveControl.getWantedX/Y/Z`. Only meaningful for move controls that implement
    /// `has_wanted`; used by goals (e.g. `Ghast.RandomFloatAroundGoal`) that decide whether to
    /// re-roll a destination based on distance to the currently wanted position.
    fn get_wanted_x(&self) -> f64 {
        0.0
    }
    fn get_wanted_y(&self) -> f64 {
        0.0
    }
    fn get_wanted_z(&self) -> f64 {
        0.0
    }
}
