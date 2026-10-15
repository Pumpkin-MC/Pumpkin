use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::EntityBase;

/// An event that occurs when a hanging entity (e.g. painting, item frame) is broken.
#[cancellable]
#[derive(Event, Clone)]
pub struct HangingBreakEvent {
    /// The hanging entity that was broken.
    pub entity: Arc<dyn EntityBase>,
    /// The entity that caused the break, if any.
    pub remover: Option<Arc<dyn EntityBase>>,
}

impl HangingBreakEvent {
    #[must_use]
    pub const fn new(entity: Arc<dyn EntityBase>, remover: Option<Arc<dyn EntityBase>>) -> Self {
        Self {
            entity,
            remover,
            cancelled: false,
        }
    }
}
