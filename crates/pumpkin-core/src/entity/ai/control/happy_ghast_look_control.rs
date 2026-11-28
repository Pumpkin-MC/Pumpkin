use crate::entity::EntityBase;
use crate::entity::ai::control::{Control, LookControlTrait};
use crate::entity::mob::Mob;
use crate::entity::passive::happy_ghast::HappyGhastEntity;
use pumpkin_util::math::vector3::Vector3;

pub struct HappyGhastLookControl {
    pub look_at_cooldown: i32,
    pub wanted_position: Option<Vector3<f64>>,
}

impl Default for HappyGhastLookControl {
    fn default() -> Self {
        Self::new()
    }
}

impl HappyGhastLookControl {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            look_at_cooldown: 0,
            wanted_position: None,
        }
    }
}

impl Control for HappyGhastLookControl {}

impl LookControlTrait for HappyGhastLookControl {
    fn wanted_position(&self) -> Option<Vector3<f64>> {
        self.wanted_position
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(ghast) = mob.cast_any().downcast_ref::<HappyGhastEntity>()
            && ghast.is_on_still_timeout()
        {
            let entity = ghast.get_entity();
            let current_yaw = entity.yaw.load();
            let snapped = (current_yaw / 90.0).round() * 90.0;
            entity.yaw.store(snapped);
            entity.head_yaw.store(snapped);
            return;
        }

        if self.look_at_cooldown > 0 {
            self.look_at_cooldown -= 1;
            if let Some(wanted) = self.wanted_position {
                let pos = mob.get_entity().pos.load();
                let dx = wanted.x - pos.x;
                let dz = wanted.z - pos.z;
                let yaw = (-f64::atan2(dx, dz).to_degrees()) as f32;
                let entity = mob.get_entity();
                entity.yaw.store(yaw);
                entity.body_yaw.store(yaw);
                entity.head_yaw.store(yaw);
                return;
            }
        }

        let entity = mob.get_entity();
        let vel = entity.velocity.load();
        let yaw = (-f64::atan2(vel.x, vel.z).to_degrees()) as f32;
        entity.yaw.store(yaw);
        entity.body_yaw.store(yaw);
    }

    fn look_at_with_range(&mut self, x: f64, y: f64, z: f64, _max_yaw: f32, _max_pitch: f32) {
        self.wanted_position = Some(Vector3::new(x, y, z));
        self.look_at_cooldown = 2;
    }
}
