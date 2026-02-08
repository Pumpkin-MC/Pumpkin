pub mod entity_damage;
pub mod entity_damage_by_entity;
pub mod entity_death;
pub mod entity_explode;
pub mod entity_regain_health;
pub mod entity_spawn;
pub mod entity_target;
pub mod entity_teleport;
pub mod food_level_change;
pub mod projectile_hit;

use pumpkin_data::entity::EntityType;

/// A trait representing events related to entities.
///
/// This trait provides a method to retrieve the entity type and ID associated with the event.
pub trait EntityEvent: Send + Sync {
    /// Retrieves the entity ID associated with the event.
    fn get_entity_id(&self) -> i32;

    /// Retrieves the entity type associated with the event.
    fn get_entity_type(&self) -> &'static EntityType;
}
