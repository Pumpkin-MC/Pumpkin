use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity targets another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTargetEvent {
    /// The ID of the targeting entity.
    pub entity_id: i32,

    /// The ID of the target entity, if any.
    pub target_id: Option<i32>,
}

impl EntityTargetEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: Option<i32>) -> Self {
        Self {
            entity_id,
            target_id,
            cancelled: false,
        }
    }
}
