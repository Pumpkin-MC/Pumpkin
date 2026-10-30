use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an area effect cloud applies its effect to entities.
#[cancellable]
#[derive(Event, Clone)]
pub struct AreaEffectCloudApplyEvent {
    /// Entity ID of the area effect cloud.
    pub entity_id: i32,
    /// List of affected entity IDs.
    pub affected_entities: Vec<i32>,
}

impl AreaEffectCloudApplyEvent {
    #[must_use]
    pub const fn new(entity_id: i32, affected_entities: Vec<i32>) -> Self {
        Self {
            entity_id,
            affected_entities,
            cancelled: false,
        }
    }
}
