use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a villager changes its profession.
#[cancellable]
#[derive(Event, Clone)]
pub struct VillagerCareerChangeEvent {
    /// The ID of the villager entity.
    pub entity_id: i32,

    /// The new profession name.
    pub profession: String,

    /// The reason for the change.
    pub reason: String,
}
