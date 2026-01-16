use crate::entity::EntityBase;
use crate::entity::ai::control::Control;
use crate::entity::mob::{Mob, MobEntity};
use pumpkin_util::math::clamp_angle;
use pumpkin_util::math::vector3::Vector3;
use std::f32::consts::PI;
use std::sync::Arc;

// Please keep the atomic values out of here!!!
#[derive(Default)]
pub struct LookControl {
    max_yaw_change: f32,
    max_pitch_change: f32,
    look_at_timer: i32,
    position: Vector3<f64>,
}

impl Control for LookControl {}

impl LookControl {
    pub fn look_at_position(&mut self, mob: &dyn Mob, position: Vector3<f64>) {
        self.look_at(mob, position.x, position.y, position.z);
    }

    pub fn look_at_entity(&mut self, mob: &dyn Mob, entity: &Arc<dyn EntityBase>) {
        let entity = entity.get_entity();
        let pos = entity.pos.load();
        self.look_at_position(mob, pos);
    }

    pub fn look_at_entity_with_range(
        &mut self,
        entity: &Arc<dyn EntityBase>,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        let entity = entity.get_entity();
        let pos = entity.pos.load();
        self.look_at_with_range(pos.x, pos.y, pos.z, max_yaw_change, max_pitch_change);
    }

    pub fn look_at(&mut self, mob: &dyn Mob, x: f64, y: f64, z: f64) {
        self.look_at_with_range(
            x,
            y,
            z,
            mob.get_max_look_yaw_change(),
            mob.get_max_look_pitch_change(),
        );
    }

    pub fn look_at_with_range(
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

    pub async fn tick(&mut self, mob: &dyn Mob) {
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

        self.clamp_head_yaw(mob).await;
    }

    fn should_stay_horizontal() -> bool {
        true
    }

    async fn clamp_head_yaw(&self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let navigator = mob_entity.navigator.lock().await;
        if !navigator.is_idle() {
            let entity = &mob_entity.living_entity.entity;
            let max_head_rotation = mob.get_max_head_rotation();
            entity.head_yaw.store(clamp_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                max_head_rotation,
            ));
        }
    }

    fn get_target_pitch(&self, mob: &MobEntity) -> Option<f32> {
        let position = self.position;
        let mob_position = mob.living_entity.entity.pos.load();
        let d = position.x - mob_position.x;
        let e = position.y - mob.living_entity.entity.get_eye_y();
        let f = position.z - mob_position.z;
        let g = (d * d + f * f).sqrt();
        if e.abs() <= 1.0E-5 && g.abs() <= 1.0E-5 {
            None
        } else {
            Some(-(e.atan2(g) as f32 * 180.0 / PI))
        }
    }

    fn get_target_yaw(&self, mob: &MobEntity) -> Option<f32> {
        let position = self.position;
        let mob_position = mob.living_entity.entity.pos.load();
        let d = position.x - mob_position.x;
        let e = position.z - mob_position.z;
        if e.abs() <= 1.0E-5 && d.abs() <= 1.0E-5 {
            None
        } else {
            Some(e.atan2(d) as f32 * 180.0 / PI - 90.0)
        }
    }
}
