use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::EntityBase;

/// An event that occurs when a hanging entity is broken by another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct HangingBreakByEntityEvent {
    /// The hanging entity that was broken.
    pub entity: Arc<dyn EntityBase>,
    /// The entity that broke the hanging entity.
    pub remover: Arc<dyn EntityBase>,
}

impl HangingBreakByEntityEvent {
    #[must_use]
    pub const fn new(entity: Arc<dyn EntityBase>, remover: Arc<dyn EntityBase>) -> Self {
        Self {
            entity,
            remover,
            cancelled: false,
        }
    }
}
