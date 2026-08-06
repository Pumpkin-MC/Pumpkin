use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use crate::entity::mob::vex::VexEntity;

/// Vanilla: `Vex.VexMoveControl`.
///
/// Unlike the ground `MoveControl`, this accelerates gently toward a raw 3D point
/// (`speedModifier * 0.05 / deltaLength`) rather than driving movement input through the
/// navigator, and always faces the vex's current combat target while moving (falling back to
/// facing the direction of travel only when there is no target).
pub struct VexMoveControl {
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    speed_modifier: f64,
    operation: Operation,
}

impl Default for VexMoveControl {
    fn default() -> Self {
        Self {
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            speed_modifier: 0.0,
            operation: Operation::Wait,
        }
    }
}

impl Control for VexMoveControl {}

impl MoveControlTrait for VexMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if self.operation != Operation::MoveTo {
            return;
        }
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let delta = Vector3::new(
            self.wanted_x - pos.x,
            self.wanted_y - pos.y,
            self.wanted_z - pos.z,
        );
        let delta_length = delta.length();
        let bb_size = entity.bounding_box.load().get_average_side_length();

        if delta_length < bb_size {
            self.operation = Operation::Wait;
            let velocity = entity.velocity.load();
            entity.set_velocity(velocity * 0.5);
            return;
        }

        let scale = self.speed_modifier * 0.05 / delta_length;
        let velocity = entity.velocity.load();
        entity.set_velocity(velocity + delta * scale);

        // Vanilla downcasts to `Vex` here since `VexMoveControl` is specific to it; Pumpkin's
        // trait object doesn't expose the mob-specific target lookup, so read it via `try_lock`
        // (this runs synchronously inside the tick loop, unlike goals which can `.await`).
        let target = mob
            .cast_any()
            .downcast_ref::<VexEntity>()
            .and_then(|vex| vex.mob_entity.target.try_lock().ok())
            .and_then(|guard| guard.clone());

        let yaw = target.map_or_else(
            || {
                let movement = entity.velocity.load();
                -f64::atan2(movement.x, movement.z).to_degrees() as f32
            },
            |target| {
                let target_pos = target.get_entity().pos.load();
                let tx = target_pos.x - pos.x;
                let tz = target_pos.z - pos.z;
                -f64::atan2(tx, tz).to_degrees() as f32
            },
        );
        entity.yaw.store(yaw);
        entity.head_yaw.store(yaw);
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted_x = x;
        self.wanted_y = y;
        self.wanted_z = z;
        self.speed_modifier = speed_modifier;
        self.operation = Operation::MoveTo;
    }
}
