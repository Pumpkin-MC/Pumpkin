use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::EntityBase;

/// An event that occurs when a generic game event is triggered in a world.
#[cancellable]
#[derive(Event, Clone)]
pub struct GenericGameEvent {
    /// The key or type of the game event.
    pub event_key: String,

    /// The position where the game event occurred.
    pub position: Vector3<f64>,

    /// The entity that caused the game event, if known.
    pub source_entity: Option<Arc<dyn EntityBase>>,
}

impl GenericGameEvent {
    #[must_use]
    pub const fn new(
        event_key: String,
        position: Vector3<f64>,
        source_entity: Option<Arc<dyn EntityBase>>,
    ) -> Self {
        Self {
            event_key,
            position,
            source_entity,
            cancelled: false,
        }
    }
}
