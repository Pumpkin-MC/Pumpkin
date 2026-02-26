use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::projectile::ProjectileHit;

/// An event that occurs when a projectile hits a block or entity.
///
/// Cancelling this event prevents the impact effects (damage, spawns, etc.)
/// but the projectile is still consumed and removed from the world.
#[cancellable]
#[derive(Event, Clone)]
pub struct ProjectileHitEvent {
    /// The entity ID of the projectile.
    pub projectile_entity_id: i32,
    /// The type of the projectile entity.
    pub projectile_type: &'static EntityType,
    /// The entity ID of the entity that launched this projectile, if any.
    pub owner_entity_id: Option<i32>,
    /// The exact world coordinates of the impact point.
    pub hit_pos: Vector3<f64>,
    /// The block position that was hit, if this was a block hit.
    pub block_pos: Option<BlockPos>,
    /// The face of the block that was hit, if this was a block hit.
    pub block_face: Option<BlockDirection>,
    /// The entity ID of the entity that was hit, if this was an entity hit.
    pub hit_entity_id: Option<i32>,
}

impl ProjectileHitEvent {
    pub fn new(
        projectile_entity_id: i32,
        projectile_type: &'static EntityType,
        owner_entity_id: Option<i32>,
        hit: &ProjectileHit,
    ) -> Self {
        let hit_pos = hit.hit_pos();
        let (block_pos, block_face, hit_entity_id) = match hit {
            ProjectileHit::Block { pos, face, .. } => (Some(*pos), Some(*face), None),
            ProjectileHit::Entity { entity, .. } => {
                (None, None, Some(entity.get_entity().entity_id))
            }
        };
        Self {
            projectile_entity_id,
            projectile_type,
            owner_entity_id,
            hit_pos,
            block_pos,
            block_face,
            hit_entity_id,
            cancelled: false,
        }
    }
}
