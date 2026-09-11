use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity targets a living entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTargetLivingEntityEvent {
    /// The ID of the targeting entity.
    pub entity_id: i32,

    /// The ID of the target living entity, if any.
    pub target_id: Option<i32>,

    /// The targeting reason.
    pub reason: String,
}
