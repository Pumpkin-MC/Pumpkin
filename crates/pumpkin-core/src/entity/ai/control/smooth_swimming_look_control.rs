use crate::entity::ai::control::{Control, LookControlTrait, look_control::LookControl};
use crate::entity::mob::Mob;
use pumpkin_util::math::wrap_degrees;

pub struct SmoothSwimmingLookControl {
    look: LookControl,
    pub max_y_rot_from_center: i32,
}

impl SmoothSwimmingLookControl {
    #[must_use]
    pub fn new(max_y_rot_from_center: i32) -> Self {
        Self {
            look: LookControl::default(),
            max_y_rot_from_center,
        }
    }
}

impl LookControlTrait for SmoothSwimmingLookControl {
    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.look
            .look_at_with_range(x, y, z, max_yaw_change, max_pitch_change);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;

        if self.look.look_at_timer > 0 {
            self.look.look_at_timer -= 1;
            if let Some(yaw) = self.look.get_target_yaw(mob_entity) {
                entity.head_yaw.store(self.change_angle(
                    entity.head_yaw.load(),
                    yaw + 20.0,
                    self.look.max_yaw_change,
                ));
            }
            if let Some(pitch) = self.look.get_target_pitch(mob_entity) {
                entity.set_pitch(self.change_angle(
                    entity.pitch.load(),
                    pitch + 10.0,
                    self.look.max_pitch_change,
                ));
            }
        } else {
            let is_idle = mob_entity.steered().navigator().is_done();
            if is_idle {
                entity.set_pitch(self.change_angle(entity.pitch.load(), 0.0, 5.0));
            }
            entity.head_yaw.store(self.change_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                self.look.max_yaw_change,
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
}

impl Control for SmoothSwimmingLookControl {}
