use std::f32::{self};

use pumpkin_util::math::vector3::Vector3;

use super::{Entity, EntityBase, living::LivingEntity};

pub struct ThrownItemEntity {
    entity: Entity,
}

impl ThrownItemEntity {
    pub fn new(entity: Entity, owner: &Entity) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y = (owner_pos.y + f64::from(owner.standing_eye_height)) - 0.1;
        entity.pos.store(owner_pos);
        Self { entity }
    }
    pub fn set_velocity_from(
        &self,
        shooter: &Entity,
        pitch: f32,
        yaw: f32,
        roll: f32,
        speed: f32,
        divergence: f32,
    ) {
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let roll_rad = (pitch + roll).to_radians();

        let x = -yaw_rad.sin() * pitch_rad.cos();
        let y = -roll_rad.sin();
        let z = yaw_rad.cos() * pitch_rad.cos();
        self.set_velocity(
            f64::from(x),
            f64::from(y),
            f64::from(z),
            f64::from(speed),
            f64::from(divergence),
        );
        let shooter_vel = shooter.velocity.load();
        self.entity
            .velocity
            .store(self.entity.velocity.load().add_raw(
                shooter_vel.x,
                if shooter.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                    0.0
                } else {
                    shooter_vel.y
                },
                shooter_vel.z,
            ));
    }
    /// The velocity and rotation will be set to the same direction.
    pub fn set_velocity(&self, x: f64, y: f64, z: f64, power: f64, uncertainty: f64) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            mode + deviation * (rand::random::<f64>() - rand::random::<f64>())
        }
        let velocity = Vector3::new(x, y, z)
            .normalize()
            .add_raw(
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
            )
            .multiply(power, power, power);
        self.entity.velocity.store(velocity);
        let len = velocity.horizontal_length();
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(len) as f32 * 57.295_776,
        );
    }
}

impl EntityBase for ThrownItemEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
}
