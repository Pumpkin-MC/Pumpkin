use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    #[default]
    Wait,
    MoveTo,
    Strafe,
    Jumping,
}

pub struct MoveControl {
    pub wanted_x: f64,
    pub wanted_y: f64,
    pub wanted_z: f64,
    pub speed_modifier: f64,
    pub strafe_forwards: f32,
    pub strafe_right: f32,
    pub operation: Operation,
}

impl Default for MoveControl {
    fn default() -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier: 0.0,
            strafe_forwards: 0.0,
            strafe_right: 0.0,
            operation: Operation::Wait,
        }
    }
}

impl Control for MoveControl {}

impl MoveControlTrait for MoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;
        if self.operation == Operation::Strafe {
            // MoveControl STRAFE: setSpeed(speedModifier * MOVEMENT_SPEED), then the raw
            // strafe components go into xxa/zza.
            living_entity.set_speed(living_entity.speed_for_modifier(self.speed_modifier));
            living_entity.movement_input.store(Vector3::new(
                f64::from(self.strafe_right),
                0.0,
                f64::from(self.strafe_forwards),
            ));
            self.operation = Operation::Wait;
        } else if self.operation == Operation::MoveTo {
            self.operation = Operation::Wait;
            let pos = entity.pos.load();
            let xd = self.wanted_x - pos.x;
            let zd = self.wanted_z - pos.z;
            let yd = self.wanted_y - pos.y;
            let dd = xd * xd + yd * yd + zd * zd;

            if dd < 2.5000003E-7 {
                living_entity
                    .movement_input
                    .store(Vector3::new(0.0, 0.0, 0.0));
                return;
            }

            let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
            entity
                .yaw
                .store(self.change_angle(entity.yaw.load(), y_rot_d, 90.0));

            living_entity.set_speed(living_entity.speed_for_modifier(self.speed_modifier));

            let step_height = living_entity.get_attribute_value(&Attributes::STEP_HEIGHT);
            let horizontal_distance_sq = xd * xd + zd * zd;
            if should_jump(
                entity.horizontal_collision.load(Ordering::Relaxed),
                entity.on_ground.load(Ordering::Relaxed),
                yd,
                step_height,
                horizontal_distance_sq,
                entity.entity_dimension.load().width as f64,
            ) {
                living_entity.jumping.store(true, Ordering::SeqCst);
                self.operation = Operation::Jumping;
            }
        } else if self.operation == Operation::Jumping {
            living_entity.set_speed(living_entity.speed_for_modifier(self.speed_modifier));

            if entity.on_ground.load(Ordering::Relaxed) {
                living_entity.jumping.store(false, Ordering::SeqCst);
                self.operation = Operation::Wait;
            }
        }

        // Navigator owns movement input while this controller waits.
    }

    fn has_wanted(&self) -> bool {
        Self::has_wanted(self)
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        Self::set_wanted_position(self, x, y, z, speed_modifier);
    }
}

fn should_jump(
    horizontal_collision: bool,
    on_ground: bool,
    vertical_delta: f64,
    step_height: f64,
    horizontal_distance_sq: f64,
    entity_width: f64,
) -> bool {
    on_ground
        && (horizontal_collision
            || (vertical_delta > step_height && horizontal_distance_sq < 1.0f64.max(entity_width)))
}

impl MoveControl {
    #[must_use]
    pub fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    #[must_use]
    pub const fn get_speed_modifier(&self) -> f64 {
        self.speed_modifier
    }

    pub fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        if self.operation != Operation::Jumping {
            self.operation = Operation::MoveTo;
        }
    }

    pub const fn strafe(&mut self, forwards: f32, right: f32) {
        self.operation = Operation::Strafe;
        self.strafe_forwards = forwards;
        self.strafe_right = right;
        self.speed_modifier = 0.25;
    }
}

#[cfg(test)]
mod tests {
    use super::should_jump;

    #[test]
    fn grounded_collision_requests_a_jump_without_a_height_delta() {
        assert!(should_jump(true, true, 0.0, 0.6, 4.0, 0.6));
        assert!(!should_jump(true, false, 0.0, 0.6, 4.0, 0.6));
    }

    #[test]
    fn nearby_higher_path_node_requests_a_jump() {
        assert!(should_jump(false, true, 1.0, 0.6, 0.5, 0.6));
        assert!(!should_jump(false, true, 1.0, 0.6, 2.0, 0.6));
    }

    /// `MoveControl` feeds `speedModifier * MOVEMENT_SPEED` into both the mob's speed and
    /// its forward input (`Mob.setSpeed` -> `setZza`), so the attribute enters the per-tick
    /// velocity twice. `travel_in_air` on a normal block (slipperiness 0.6) contributes
    /// `speed * 0.21600002 / 0.6^3 == speed`, and horizontal friction is `0.6 * 0.91`.
    #[test]
    fn walking_mob_terminal_speed_matches_the_attribute_squared() {
        // Terminal velocity of `v += input * factor; v *= friction`.
        fn terminal(attribute: f64, speed_modifier: f64) -> f64 {
            let speed = speed_modifier * attribute;
            let per_tick = speed * (speed * 0.216_000_02 / 0.216);
            per_tick / (1.0 - 0.6 * 0.91)
        }

        // Zombie MOVEMENT_SPEED is 0.23 and spider's is 0.3 in pumpkin-data's generated
        // entity attributes. Chasing goals use a speed modifier of 1.0.
        let zombie = terminal(0.23, 1.0) * 20.0;
        let spider = terminal(0.3, 1.0) * 20.0;
        assert!((zombie - 2.331).abs() < 0.01, "zombie {zombie} blocks/s");
        assert!((spider - 3.965).abs() < 0.01, "spider {spider} blocks/s");
        // Both must stay below a walking player (4.317 blocks/s).
        assert!(zombie < 4.317 && spider < 4.317);
    }
}
