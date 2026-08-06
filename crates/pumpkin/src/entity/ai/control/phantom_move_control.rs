use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use crate::entity::mob::phantom::PhantomEntity;
use pumpkin_util::math::{vector3::Vector3, wrap_degrees};

/// Port of `Phantom.PhantomMoveControl` (`Phantom.java:385-431`).
///
/// Unlike `MoveControl`/`VexMoveControl`, this never gates on an `Operation`: vanilla's
/// `tick()` runs unconditionally every tick and reads `moveTargetPoint` straight off the
/// entity, so `has_wanted`/`set_wanted_position` are unused here (they exist only to satisfy
/// the `MoveControlTrait` interface other goals may query).
pub struct PhantomMoveControl {
    speed: f32,
}

impl Default for PhantomMoveControl {
    fn default() -> Self {
        Self { speed: 0.1 }
    }
}

impl Control for PhantomMoveControl {}

impl MoveControlTrait for PhantomMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = mob.cast_any().downcast_ref::<PhantomEntity>() else {
            return;
        };
        let entity = &mob.get_mob_entity().living_entity.entity;

        if entity
            .horizontal_collision
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            entity.yaw.store(entity.yaw.load() + 180.0);
            self.speed = 0.1;
        }

        let target = phantom.move_target_point();
        let pos = entity.pos.load();
        let mut tdx = target.x - pos.x;
        let tdy = target.y - pos.y;
        let mut tdz = target.z - pos.z;
        let sd = tdx.hypot(tdz);

        if sd.abs() <= 1.0e-5 {
            return;
        }

        let y_relative_scale = 1.0 - (tdy * 0.7).abs() / sd;
        tdx *= y_relative_scale;
        tdz *= y_relative_scale;
        let sd = tdx.hypot(tdz);
        let sd2 = (tdx * tdx + tdz * tdz + tdy * tdy).sqrt();

        let prev_yaw = entity.yaw.load();
        let angle = f64::atan2(tdz, tdx) as f32;
        let a = wrap_degrees(entity.yaw.load() + 90.0);
        let b = wrap_degrees(angle.to_degrees());
        let new_yaw = self.change_angle(a, b, 4.0) - 90.0;
        entity.yaw.store(new_yaw);
        entity.body_yaw.store(entity.yaw.load());

        if degrees_difference_abs(prev_yaw, entity.yaw.load()) < 3.0 {
            self.speed = approach(self.speed, 1.8, 0.005 * (1.8 / self.speed));
        } else {
            self.speed = approach(self.speed, 0.2, 0.025);
        }

        let x_rot_d = -(f64::atan2(-tdy, sd) as f32).to_degrees();
        entity.set_pitch(x_rot_d);

        let move_angle = entity.yaw.load() + 90.0;
        let txd =
            f64::from(self.speed) * f64::from(move_angle.to_radians().cos()) * (tdx / sd2).abs();
        let tzd =
            f64::from(self.speed) * f64::from(move_angle.to_radians().sin()) * (tdz / sd2).abs();
        let tyd = f64::from(self.speed) * f64::from(x_rot_d.to_radians().sin()) * (tdy / sd2).abs();

        let movement = entity.velocity.load();
        let wanted = Vector3::new(txd, tyd, tzd);
        entity.set_velocity(movement + (wanted - movement).multiply(0.2, 0.2, 0.2));
    }
}

fn degrees_difference_abs(a: f32, b: f32) -> f32 {
    wrap_degrees(a - b).abs()
}

fn approach(value: f32, target: f32, max_step: f32) -> f32 {
    let max_step = max_step.abs();
    if value < target {
        (value + max_step).min(target)
    } else {
        (value - max_step).max(target)
    }
}
