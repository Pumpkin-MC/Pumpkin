pub mod damage;
pub mod death;
pub mod spawn;

use std::sync::Arc;

use crate::entity::EntityBase;

/// A trait representing events related to entities.
pub trait EntityEvent: Send + Sync {
    /// Retrieves the UUID of the entity associated with the event.
    fn get_entity_uuid(&self) -> uuid::Uuid;

    /// Attempts to retrieve a reference to the entity if available.
    fn get_entity(&self) -> Option<Arc<dyn EntityBase>> {
        None
    }
}
