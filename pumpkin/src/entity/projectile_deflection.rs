use crate::entity::EntityBase;
use std::sync::atomic::Ordering;

#[derive(Clone, Copy)]
pub enum ProjectileDeflectionType {
    None,
    Simple,
    Redirected,
    TransferVelocityDirection,
}

impl ProjectileDeflectionType {
    pub fn deflect(&self, projectile: &mut dyn EntityBase, hit_entity: Option<&dyn EntityBase>) {
        match self {
            Self::None => {}
            Self::Simple => {
                let vel = 170.0 + rand::random::<f32>() * 20.0;

                let current_velocity = projectile
                    .get_entity()
                    .velocity
                    .load()
                    .multiply(-0.5, -0.5, -0.5);

                let entity = projectile.get_entity();
                entity.velocity.store(current_velocity);

                let yaw = entity.yaw.load() + vel;
                let pitch = entity.pitch.load();
                entity.set_rotation(yaw, pitch);
                // TODO: Add entity.lastYaw += vel
                entity.velocity_dirty.store(true, Ordering::Relaxed);
            }
            Self::Redirected => {
                if let Some(hit_entity) = hit_entity {
                    let rotation_vector = hit_entity.get_entity().rotation();

                    let entity = projectile.get_entity();
                    entity.velocity.store(rotation_vector.to_f64());
                    entity.velocity_dirty.store(true, Ordering::Relaxed);
                }
            }
            Self::TransferVelocityDirection => {
                if let Some(hit_entity) = hit_entity {
                    let hit_velocity = hit_entity.get_entity().velocity.load().normalize();

                    let entity = projectile.get_entity();
                    entity.velocity.store(hit_velocity);
                    entity.velocity_dirty.store(true, Ordering::Relaxed);
                }
            }
        }
    }
}
