use crate::entity::ai::control::{Control, LookControlTrait};
use crate::entity::mob::{Mob, MobEntity};
use pumpkin_util::math::clamp_angle;
use pumpkin_util::math::vector3::Vector3;

// Please keep the atomic values out of here!!!
#[derive(Default)]
pub struct LookControl {
    pub(super) max_yaw_change: f32,
    pub(super) max_pitch_change: f32,
    pub(super) look_at_timer: i32,
    pub(super) position: Vector3<f64>,
}

impl Control for LookControl {}

impl LookControl {
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
        let entity = mob.get_entity();
        if Self::should_stay_horizontal() {
            entity.set_pitch(0.0);
        }

        if self.look_at_timer > 0 {
            self.look_at_timer -= 1;
            if let Some(yaw) = self.get_target_yaw(mob.get_mob_entity()) {
                entity.head_yaw.store(self.change_angle(
                    entity.head_yaw.load(),
                    yaw,
                    self.max_yaw_change,
                ));
            }
            if let Some(pitch) = self.get_target_pitch(mob.get_mob_entity()) {
                entity.set_pitch(self.change_angle(
                    entity.pitch.load(),
                    pitch,
                    self.max_pitch_change,
                ));
            }
        } else {
            entity.head_yaw.store(self.change_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                10.0,
            ));
        }

        Self::clamp_head_yaw(mob);
    }

    const fn should_stay_horizontal() -> bool {
        true
    }

    fn clamp_head_yaw(mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let steered = mob_entity.steered();
        let navigator = steered.navigator();
        if !navigator.is_done() {
            let entity = &mob_entity.living_entity.entity;
            let max_head_rotation = mob.get_max_head_rotation();
            entity.head_yaw.store(clamp_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                max_head_rotation,
            ));
        }
    }

    pub(super) fn get_target_pitch(&self, mob: &MobEntity) -> Option<f32> {
        let position = self.position;
        let mob_position = mob.living_entity.entity.pos.load();
        let d = position.x - mob_position.x;
        let e = position.y - mob.living_entity.entity.get_eye_y();
        let f = position.z - mob_position.z;
        let g = d.hypot(f);
        if e.abs() <= 1.0E-5 && g.abs() <= 1.0E-5 {
            None
        } else {
            Some(-(e.atan2(g) as f32).to_degrees())
        }
    }

    pub(super) fn get_target_yaw(&self, mob: &MobEntity) -> Option<f32> {
        let position = self.position;
        let mob_position = mob.living_entity.entity.pos.load();
        let d = position.x - mob_position.x;
        let e = position.z - mob_position.z;
        if e.abs() <= 1.0E-5 && d.abs() <= 1.0E-5 {
            None
        } else {
            Some((e.atan2(d) as f32).to_degrees() - 90.0)
        }
    }
}

impl LookControlTrait for LookControl {
    fn tick(&mut self, mob: &dyn Mob) {
        Self::tick(self, mob);
    }

    fn look_at_with_range(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        Self::look_at_with_range(self, x, y, z, max_yaw_change, max_pitch_change);
    }
}
