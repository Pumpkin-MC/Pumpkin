use pumpkin_util::math::vector3::Vector3;

use super::{PandaEntity, as_panda};
use crate::entity::ai::control::{Control, MoveControlTrait, move_control::MoveControl};
use crate::entity::mob::Mob;

/// Vanilla `PandaMoveControl`: the panda only steers while it can perform an action, so a panda
/// that sits, eats, rolls or lies on its back keeps its wanted position but does not walk.
#[derive(Default)]
pub struct PandaMoveControl {
    inner: MoveControl,
}

impl Control for PandaMoveControl {}

impl MoveControlTrait for PandaMoveControl {
    /// Vanilla `PandaMoveControl.tick`: steers only while the panda can perform an action.
    fn tick(&mut self, mob: &dyn Mob) {
        if as_panda(mob).is_none_or(PandaEntity::can_perform_action) {
            self.inner.tick(mob);
        }
    }

    /// Vanilla `MoveControl.setWantedPosition`.
    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.inner.set_wanted_position(x, y, z, speed_modifier);
    }

    /// Vanilla `MoveControl.strafe`.
    fn strafe(&mut self, forwards: f32, right: f32) {
        self.inner.strafe(forwards, right);
    }

    /// Vanilla `MoveControl.hasWanted`.
    fn has_wanted(&self) -> bool {
        self.inner.has_wanted()
    }

    /// Vanilla `MoveControl.getSpeedModifier`.
    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
    }

    /// Vanilla `MoveControl.getWantedX/Y/Z`.
    fn get_wanted_position(&self) -> Vector3<f64> {
        MoveControlTrait::get_wanted_position(&self.inner)
    }
}
