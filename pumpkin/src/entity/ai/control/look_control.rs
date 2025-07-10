use crate::entity::EntityBase;
use crate::entity::ai::control::Control;
use crate::entity::mob::MobEntity;
use crossbeam::atomic::AtomicCell;
use pumpkin_util::math::clamp_angle;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Default)]
pub struct LookControl {
    max_yaw_change: AtomicCell<f32>,
    max_pitch_change: AtomicCell<f32>,
    look_at_timer: AtomicI32,
    position: AtomicCell<Vector3<f64>>,
}

impl Control for LookControl {}

impl LookControl {
    pub fn look_at_position(&self, mob: &MobEntity, position: Vector3<f64>) {
        self.look_at(mob, position.x, position.y, position.z);
    }

    pub fn look_at_entity(&self, mob: &MobEntity, entity: &Arc<dyn EntityBase>) {
        let entity = entity.get_entity();
        let pos = entity.pos.load();
        self.look_at_position(mob, pos);
    }

    pub fn look_at_entity_with_range(
        &self,
        entity: &Arc<dyn EntityBase>,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        let entity = entity.get_entity();
        let pos = entity.pos.load();
        self.look_at_with_range(pos.x, pos.y, pos.z, max_yaw_change, max_pitch_change);
    }

    pub fn look_at(&self, mob: &MobEntity, x: f64, y: f64, z: f64) {
        let (max_look_yaw_change, max_look_pitch_change) = mob.mob.as_ref().map_or_else(
            || (self.max_yaw_change.load(), self.max_pitch_change.load()),
            |mob| {
                (
                    mob.get_max_look_yaw_change() as f32,
                    mob.get_max_look_pitch_change() as f32,
                )
            },
        );
        self.look_at_with_range(x, y, z, max_look_yaw_change, max_look_pitch_change);
    }

    pub fn look_at_with_range(
        &self,
        x: f64,
        y: f64,
        z: f64,
        max_yaw_change: f32,
        max_pitch_change: f32,
    ) {
        self.position.store(Vector3::new(x, y, z));
        self.max_yaw_change.store(max_yaw_change);
        self.max_pitch_change.store(max_pitch_change);
        self.look_at_timer.store(2, Relaxed);
    }

    pub async fn tick(&self, mob: &MobEntity) {
        let entity = mob.get_entity();
        if Self::should_stay_horizontal() {
            entity.set_pitch(0.0);
        }

        if self.look_at_timer.load(Relaxed) > 0 {
            self.look_at_timer.fetch_sub(1, Relaxed);
            if let Some(yaw) = self.get_target_yaw(mob) {
                entity.head_yaw.store(self.change_angle(
                    entity.head_yaw.load(),
                    yaw,
                    self.max_yaw_change.load(),
                ));
            }
            if let Some(pitch) = self.get_target_pitch(mob) {
                entity.set_pitch(self.change_angle(
                    entity.pitch.load(),
                    pitch,
                    self.max_pitch_change.load(),
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

    async fn clamp_head_yaw(&self, mob: &MobEntity) {
        let navigator = mob.navigator.lock().await;
        if !navigator.is_idle() {
            let entity = &mob.living_entity.entity;
            let max_head_rotation = mob
                .mob
                .as_ref()
                .map_or(90.0, |mob| mob.get_max_head_rotation() as f32);
            entity.head_yaw.store(clamp_angle(
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                max_head_rotation,
            ));
        }
    }

    fn get_target_pitch(&self, mob: &MobEntity) -> Option<f32> {
        let position = self.position.load();
        let mob_position = mob.living_entity.entity.pos.load();
        let d = position.x - mob_position.x;
        let e = position.y - mob.living_entity.entity.get_eye_y();
        let f = position.z - mob_position.z;
        let g = (d * d + f * f).sqrt();
        if e.abs() <= 1.0E-5 && g.abs() <= 1.0E-5 {
            None
        } else {
            Some(-(e.atan2(g) as f32 * 180.0 / std::f32::consts::PI))
        }
    }

    fn get_target_yaw(&self, mob: &MobEntity) -> Option<f32> {
        let position = self.position.load();
        let mob_position = mob.living_entity.entity.pos.load();
        let d = position.x - mob_position.x;
        let e = position.z - mob_position.z;
        if e.abs() <= 1.0E-5 && d.abs() <= 1.0E-5 {
            None
        } else {
            Some(e.atan2(d) as f32 * 180.0 / std::f32::consts::PI - 90.0)
        }
    }
}
