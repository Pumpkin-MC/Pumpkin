use crate::entity::{Entity, EntityBase};
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileDeflection {
    None,
    /// Bounces back with half the speed, turned 170 to 190 degrees.
    Reverse,
    /// Flies where the deflecting entity looks.
    AimDeflect,
    /// Takes over the deflecting entity's direction of movement.
    MomentumDeflect,
    /// A breeze plays its deflect sound, then reverses the projectile.
    Breeze,
}

impl ProjectileDeflection {
    fn apply(
        self,
        projectile: &dyn EntityBase,
        entity: Option<&dyn EntityBase>,
        power: Vector3<f64>,
    ) {
        let projectile_entity = projectile.get_entity();
        match self {
            Self::None => return,
            Self::Breeze => {
                if let Some(entity) = entity {
                    entity.play_entity_sound(Sound::EntityBreezeDeflect, 1.0, 1.0);
                }
                Self::Reverse.apply(projectile, entity, power);
                return;
            }
            Self::Reverse => {
                let rotation = rand::random::<f32>().mul_add(20.0, 170.0);
                let velocity = projectile_entity.velocity.load();
                projectile_entity.velocity.store(Vector3::new(
                    -power.x * 0.5 * velocity.x,
                    -power.y * 0.5 * velocity.y,
                    -power.z * 0.5 * velocity.z,
                ));
                projectile_entity.set_rotation(
                    projectile_entity.yaw.load() + rotation,
                    projectile_entity.pitch.load(),
                );
            }
            Self::AimDeflect => {
                let Some(entity) = entity else {
                    return;
                };
                let look = entity.get_entity().rotation().to_f64();
                projectile_entity.velocity.store(Vector3::new(
                    look.x * power.x,
                    look.y * power.y,
                    look.z * power.z,
                ));
            }
            Self::MomentumDeflect => {
                let Some(entity) = entity else {
                    return;
                };
                let movement = entity.get_entity().velocity.load().normalize();
                projectile_entity.velocity.store(Vector3::new(
                    movement.x * power.x,
                    movement.y * power.y,
                    movement.z * power.z,
                ));
            }
        }
        projectile_entity
            .velocity_dirty
            .store(true, Ordering::Relaxed);
    }
}

/// The deflection of an entity that has no special rule: breezes reverse projectiles.
#[must_use]
pub fn default_deflection(entity: &Entity) -> ProjectileDeflection {
    if entity
        .entity_type
        .has_tag(&tag::EntityType::MINECRAFT_DEFLECTS_PROJECTILES)
    {
        ProjectileDeflection::Reverse
    } else {
        ProjectileDeflection::None
    }
}

/// Deflects `projectile` and hands it to `new_owner`. False when the projectile refused, e.g. a
/// wind charge that was deflected moments ago.
pub fn deflect(
    projectile: &dyn EntityBase,
    deflection: ProjectileDeflection,
    deflecting_entity: Option<&dyn EntityBase>,
    new_owner: Option<Uuid>,
    by_attack: bool,
    power: Vector3<f64>,
) -> bool {
    if !projectile.can_be_deflected() {
        return false;
    }
    deflection.apply(projectile, deflecting_entity, power);
    if let Some(owner) = projectile.get_projectile_owner() {
        owner.set_uuid(new_owner);
    }
    projectile.on_deflection(by_attack);
    true
}
