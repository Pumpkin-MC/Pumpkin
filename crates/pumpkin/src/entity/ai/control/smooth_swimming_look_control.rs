use crate::entity::ai::control::Control;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;

pub struct SmoothSwimmingLookControl {
    pub max_yaw_change: f32,
    pub max_pitch_change: f32,
    pub look_at_timer: i32,
    pub position: Vector3<f64>,
    pub max_y_rot_from_center: i32,
}

impl SmoothSwimmingLookControl {
    #[must_use]
    pub const fn new(max_y_rot_from_center: i32) -> Self {
        Self {
            max_yaw_change: 0.0,
            max_pitch_change: 0.0,
            look_at_timer: 0,
            position: Vector3::new(0.0, 0.0, 0.0),
            max_y_rot_from_center,
        }
    }

    pub const fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.position = Vector3::new(x, y, z);
        self.max_yaw_change = max_yaw_change;
        self.max_pitch_change = max_pitch_change;
        self.look_at_timer = 2;
    }

    pub fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;

        if self.look_at_timer > 0 {
            self.look_at_timer -= 1;
            if let Some(yaw) = self.get_target_yaw(mob) {
                entity.head_yaw.store(self.change_angle(
                    entity.head_yaw.load(),
                    yaw + 20.0,
                    self.max_yaw_change,
                ));
            }
            if let Some(pitch) = self.get_target_pitch(mob) {
                entity.set_pitch(self.change_angle(
                    entity.pitch.load(),
                    pitch + 10.0,
                    self.max_pitch_change,
                ));
            }
        } else {
            let is_idle = mob_entity
                .navigator
                .try_lock()
                .is_ok_and(|navigator| navigator.is_idle());
            if is_idle {
                entity.set_pitch(self.change_angle(entity.pitch.load(), 0.0, 5.0));
            }
            entity.head_yaw.store(self.change_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                self.max_yaw_change,
            ));
        }

        let head_diff_body = wrap_degrees(entity.head_yaw.load() - entity.body_yaw.load());
        let max_rot = self.max_y_rot_from_center as f32;
        if head_diff_body < -max_rot {
            let body_yaw = entity.body_yaw.load();
            entity.body_yaw.store(body_yaw - 4.0);
        } else if head_diff_body > max_rot {
            let body_yaw = entity.body_yaw.load();
            entity.body_yaw.store(body_yaw + 4.0);
        }
    }

    fn get_target_pitch(&self, mob: &dyn Mob) -> Option<f32> {
        let position = self.position;
        let mob_position = mob.get_entity().pos.load();
        let d = position.x - mob_position.x;
        let e = position.y - mob.get_entity().get_eye_y();
        let f = position.z - mob_position.z;
        let g = d.hypot(f);
        if e.abs() <= 1.0E-5 && g.abs() <= 1.0E-5 {
            None
        } else {
            Some(-(e.atan2(g) as f32).to_degrees())
        }
    }

    fn get_target_yaw(&self, mob: &dyn Mob) -> Option<f32> {
        let position = self.position;
        let mob_position = mob.get_entity().pos.load();
        let d = position.x - mob_position.x;
        let e = position.z - mob_position.z;
        if e.abs() <= 1.0E-5 && d.abs() <= 1.0E-5 {
            None
        } else {
            Some((e.atan2(d) as f32).to_degrees() - 90.0)
        }
    }
}

impl Control for SmoothSwimmingLookControl {}
