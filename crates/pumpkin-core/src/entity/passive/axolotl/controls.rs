use pumpkin_util::math::vector3::Vector3;

use super::as_axolotl;
use crate::entity::ai::control::{
    Control, LookControlTrait, MoveControlTrait,
    smooth_swimming_look_control::SmoothSwimmingLookControl,
    smooth_swimming_move_control::SmoothSwimmingMoveControl,
};
use crate::entity::mob::Mob;

const MAX_TURN_X: i32 = 85;
const MAX_TURN_Y: i32 = 10;
const IN_WATER_SPEED_MODIFIER: f32 = 0.1;
const OUTSIDE_WATER_SPEED_MODIFIER: f32 = 0.5;
const MAX_Y_ROT_FROM_CENTER: i32 = 20;

fn is_playing_dead(mob: &dyn Mob) -> bool {
    as_axolotl(mob).is_some_and(super::AxolotlEntity::is_playing_dead)
}

pub struct AxolotlMoveControl {
    inner: SmoothSwimmingMoveControl,
}

impl Default for AxolotlMoveControl {
    fn default() -> Self {
        Self {
            inner: SmoothSwimmingMoveControl::new(
                MAX_TURN_X,
                MAX_TURN_Y,
                IN_WATER_SPEED_MODIFIER,
                OUTSIDE_WATER_SPEED_MODIFIER,
                false,
            ),
        }
    }
}

impl Control for AxolotlMoveControl {}

impl MoveControlTrait for AxolotlMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if !is_playing_dead(mob) {
            self.inner.tick(mob);
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

    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        self.inner.get_wanted_position()
    }
}

pub struct AxolotlLookControl {
    inner: SmoothSwimmingLookControl,
}

impl Default for AxolotlLookControl {
    fn default() -> Self {
        Self {
            inner: SmoothSwimmingLookControl::new(MAX_Y_ROT_FROM_CENTER),
        }
    }
}

impl Control for AxolotlLookControl {}

impl LookControlTrait for AxolotlLookControl {
    fn wanted_position(&self) -> Option<Vector3<f64>> {
        self.inner.wanted_position()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if !is_playing_dead(mob) {
            self.inner.tick(mob);
        }
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.inner
            .look_at_with_range(x, y, z, max_yaw_change, max_pitch_change);
    }
}
