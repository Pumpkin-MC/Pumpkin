use pumpkin_data::attributes::Attributes;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;

/// Vanilla: `Ghast.GhastMoveControl` (Ghast.java:228-323).
///
/// Ghast constructs this with `careful = false` (Ghast.java:52), so the `careful`-only branch
/// of `blockTraversalPossible` (fluid/`HAPPY_GHAST_AVOIDS` checks, Ghast.java:302-319) never
/// runs for an actual `Ghast` and is not ported here.
///
/// `canReach`'s precise per-block `collidedWithShapeMovingFrom` sweep (Ghast.java:262-283) has
/// no equivalent in Pumpkin's collision system; `can_reach` below samples the straight-line
/// path at roughly 1-block intervals and rejects it if any sampled block is solid, which
/// approximates the same "don't fly a path through walls" intent without exact block shapes.
pub struct GhastMoveControl {
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    operation: Operation,
    float_duration: i32,
}

impl Default for GhastMoveControl {
    fn default() -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            operation: Operation::Wait,
            float_duration: 0,
        }
    }
}

impl Control for GhastMoveControl {}

impl MoveControlTrait for GhastMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if self.operation != Operation::MoveTo {
            return;
        }

        let (decremented, should_move) = decrement_float_duration(self.float_duration);
        self.float_duration = decremented;
        if !should_move {
            return;
        }
        self.float_duration += mob.get_random().random_range(0..5) + 2;

        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let travel = Vector3::new(
            self.wanted_x - pos.x,
            self.wanted_y - pos.y,
            self.wanted_z - pos.z,
        );

        if can_reach(mob, travel) {
            let flying_speed = mob
                .get_mob_entity()
                .living_entity
                .get_attribute_value(&Attributes::FLYING_SPEED);
            let scale = flying_speed * 5.0 / 3.0;
            let velocity = entity.velocity.load();
            // Vanilla: `set_velocity` also broadcasts `CEntityVelocity` (mirrors
            // `VexMoveControl`'s use of `Entity#set_velocity` over a bare store).
            entity.set_velocity(velocity.add(&(travel.normalize() * scale)));
        } else {
            self.operation = Operation::Wait;
        }
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, _speed_modifier: f64) {
        // Vanilla's `GhastMoveControl.tick` never reads `speedModifier` -- it always scales by
        // `FLYING_SPEED` (Ghast.java:253) -- so unlike the base `MoveControl` it is not stored.
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.operation = Operation::MoveTo;
    }

    fn get_wanted_x(&self) -> f64 {
        self.wanted_x
    }

    fn get_wanted_y(&self) -> f64 {
        self.wanted_y
    }

    fn get_wanted_z(&self) -> f64 {
        self.wanted_z
    }
}

/// Vanilla: `if (this.floatDuration-- <= 0)` (Ghast.java:247) is a post-decrement -- the gate
/// tests the value *before* decrementing, and the caller's reassignment adds onto the
/// already-decremented (possibly negative) remainder rather than overwriting it. Returns the
/// decremented duration and whether the move should run this tick.
const fn decrement_float_duration(current: i32) -> (i32, bool) {
    (current - 1, current <= 0)
}

fn can_reach(mob: &dyn Mob, travel: Vector3<f64>) -> bool {
    let distance = travel.length();
    if distance < 1.0e-6 {
        return true;
    }

    let entity = &mob.get_mob_entity().living_entity.entity;
    let start = entity.pos.load();
    let world = entity.world.load();
    let steps = distance.ceil().max(1.0) as i32;

    for i in 0..=steps {
        let t = f64::from(i) / f64::from(steps);
        let block_pos = BlockPos::new(
            (start.x + travel.x * t).floor() as i32,
            (start.y + travel.y * t).floor() as i32,
            (start.z + travel.z * t).floor() as i32,
        );
        // `World::get_block_state` falls back to `Block::AIR` for an unloaded position rather
        // than blocking or panicking (world/mod.rs:5578-5585), so a sample that strays into an
        // unloaded chunk is simply treated as passable here.
        let state = world.get_block_state(&block_pos);
        if !state.is_air() && state.is_solid_block() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{GhastMoveControl, decrement_float_duration};
    use crate::entity::ai::control::MoveControlTrait;

    #[test]
    fn decrement_gate_gates_on_the_pre_decrement_value() {
        assert_eq!(decrement_float_duration(2), (1, false));
        assert_eq!(decrement_float_duration(1), (0, false));
        assert_eq!(decrement_float_duration(0), (-1, true));
        assert_eq!(decrement_float_duration(-3), (-4, true));
    }

    #[test]
    fn wanted_position_round_trips_and_ignores_speed_modifier() {
        let mut control = GhastMoveControl::default();
        assert!(!control.has_wanted());
        control.set_wanted_position(1.0, 2.0, 3.0, 0.25);
        assert!(control.has_wanted());
        assert!((control.get_wanted_x() - 1.0).abs() < f64::EPSILON);
        assert!((control.get_wanted_y() - 2.0).abs() < f64::EPSILON);
        assert!((control.get_wanted_z() - 3.0).abs() < f64::EPSILON);
    }
}
