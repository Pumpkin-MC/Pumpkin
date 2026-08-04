//! Port of `Phantom.PhantomCircleAroundAnchorGoal` (`Phantom.java:311-373`).

use std::sync::Weak;

use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{cos, sin};
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::mob::phantom::{AttackPhase, PhantomEntity};

pub struct PhantomCircleAroundAnchorGoal {
    phantom: Weak<PhantomEntity>,
    angle: f32,
    distance: f32,
    height: f32,
    clockwise: f32,
}

impl PhantomCircleAroundAnchorGoal {
    #[must_use]
    pub const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            angle: 0.0,
            distance: 0.0,
            height: 0.0,
            clockwise: 1.0,
        }
    }

    fn select_next(&mut self, phantom: &PhantomEntity) {
        let anchor = phantom.anchor_point().unwrap_or_else(|| {
            let block_pos = phantom.mob_entity.living_entity.entity.block_pos.load();
            phantom.set_anchor_point(Some(block_pos));
            block_pos
        });

        self.angle = next_circle_angle(self.angle, self.clockwise);
        let anchor_lower_corner = Vector3::new(
            f64::from(anchor.0.x),
            f64::from(anchor.0.y),
            f64::from(anchor.0.z),
        );
        phantom.set_move_target_point(circle_move_target(
            anchor_lower_corner,
            self.distance,
            self.height,
            self.angle,
        ));
    }

    fn touching_target(phantom: &PhantomEntity) -> bool {
        let target = phantom.move_target_point();
        let pos = phantom.mob_entity.living_entity.entity.pos.load();
        target.squared_distance_to_vec(&pos) < 4.0
    }
}

/// Vanilla: `this.angle = this.angle + this.clockwise * 15.0F * ((float)Math.PI / 180.0F)`.
#[must_use]
pub fn next_circle_angle(current_angle: f32, clockwise: f32) -> f32 {
    current_angle + clockwise * 15.0f32.to_radians()
}

/// Vanilla `selectNext`'s target point:
/// `Vec3.atLowerCornerOf(anchorPoint).add(distance * cos(angle), -4.0F + height, distance * sin(angle))`.
#[must_use]
pub fn circle_move_target(
    anchor_lower_corner: Vector3<f64>,
    distance: f32,
    height: f32,
    angle: f32,
) -> Vector3<f64> {
    Vector3::new(
        anchor_lower_corner.x + f64::from(distance * cos(angle)),
        anchor_lower_corner.y + f64::from(-4.0 + height),
        anchor_lower_corner.z + f64::from(distance * sin(angle)),
    )
}

impl Goal for PhantomCircleAroundAnchorGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return false;
            };
            let has_target = phantom.mob_entity.target.lock().await.is_some();
            let _ = mob;
            !has_target || phantom.attack_phase() == AttackPhase::Circle
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            let mut random = mob.get_random();
            self.distance = 5.0 + random.random::<f32>() * 10.0;
            self.height = -4.0 + random.random::<f32>() * 9.0;
            self.clockwise = if random.random_bool(0.5) { 1.0 } else { -1.0 };
            self.select_next(&phantom);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return;
            };
            let mut random = mob.get_random();

            if random.random_range(0..self.get_tick_count(350).max(1)) == 0 {
                self.height = -4.0 + random.random::<f32>() * 9.0;
            }

            if random.random_range(0..self.get_tick_count(250).max(1)) == 0 {
                self.distance += 1.0;
                if self.distance > 15.0 {
                    self.distance = 5.0;
                    self.clockwise = -self.clockwise;
                }
            }

            if random.random_range(0..self.get_tick_count(450).max(1)) == 0 {
                self.angle = random.random::<f32>() * 2.0 * std::f32::consts::PI;
                self.select_next(&phantom);
            }

            if Self::touching_target(&phantom) {
                self.select_next(&phantom);
            }

            let entity = &phantom.mob_entity.living_entity.entity;
            let pos = entity.pos.load();
            let block_pos = entity.block_pos.load();
            let world = entity.world.load_full();
            let target = phantom.move_target_point();

            if target.y < pos.y && !world.get_block_state(&block_pos.down()).is_air() {
                self.height = self.height.max(1.0);
                self.select_next(&phantom);
            }

            if target.y > pos.y && !world.get_block_state(&block_pos.up()).is_air() {
                self.height = self.height.min(-1.0);
                self.select_next(&phantom);
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_advances_by_fifteen_degrees_in_radians() {
        let next = next_circle_angle(0.0, 1.0);
        assert!((next - 15.0f32.to_radians()).abs() < 1e-6);
    }

    #[test]
    fn clockwise_sign_flips_the_direction_of_travel() {
        let cw = next_circle_angle(0.0, 1.0);
        let ccw = next_circle_angle(0.0, -1.0);
        assert!((cw + ccw).abs() < 1e-6);
    }

    #[test]
    fn move_target_y_offset_range_matches_vanilla() {
        // height in [-4.0, 5.0) => y offset (-4.0 + height) in [-8.0, 1.0)
        let anchor = Vector3::new(0.0, 100.0, 0.0);
        let low = circle_move_target(anchor, 5.0, -4.0, 0.0);
        let high = circle_move_target(anchor, 5.0, 5.0 - 1e-4, 0.0);
        assert!((low.y - (100.0 - 8.0)).abs() < 1e-6);
        assert!(high.y < 101.0);
        assert!(high.y > 100.0);
    }

    #[test]
    fn touching_target_threshold_is_four_blocks_squared() {
        let target = Vector3::new(0.0, 0.0, 0.0);
        let just_inside = Vector3::new(1.9, 0.0, 0.0);
        let just_outside = Vector3::new(2.1, 0.0, 0.0);
        assert!(target.squared_distance_to_vec(&just_inside) < 4.0);
        assert!(target.squared_distance_to_vec(&just_outside) >= 4.0);
    }
}
