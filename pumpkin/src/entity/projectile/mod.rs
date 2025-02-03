use pumpkin_util::math::vector3::Vector3;

use super::{living::LivingEntity, Entity, EntityBase};

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

    pub fn set_velocity_shooter_rot(&self, shooter: &Entity, power: f32, divergence: f32) {
        self.set_velocity_shooter(shooter, shooter.rotation_vec() * power, divergence);
    }

    pub fn set_velocity_shooter(&self, shooter: &Entity, velocity: Vector3<f32>, divergence: f32) {
        self.set_velocity_unstable(velocity.to_f64(), f64::from(divergence));
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

    /// Velocity will be set a bit randomly
    pub fn set_velocity_unstable(&self, velocity: Vector3<f64>, uncertainty: f64) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            mode + deviation * (rand::random::<f64>() - rand::random::<f64>())
        }
        let velocity = velocity.add_raw(
            next_triangular(0.0, 0.017_227_5 * uncertainty),
            next_triangular(0.0, 0.017_227_5 * uncertainty),
            next_triangular(0.0, 0.017_227_5 * uncertainty),
        );
        self.set_velocity(velocity);
    }

    /// Velocity will be set normally
    pub fn set_velocity(&self, velocity: Vector3<f64>) {
        self.entity.velocity.store(velocity);
        self.entity.set_rotation_vec(velocity.as_f32());
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
