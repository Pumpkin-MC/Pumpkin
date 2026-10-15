use pumpkin_macros::{Event, cancellable};

/// An event that occurs when two item entities merge.
#[cancellable]
#[derive(Event, Clone)]
pub struct ItemMergeEvent {
    /// The ID of the primary item entity.
    pub entity_id: i32,

    /// The ID of the target item entity being merged into the primary.
    pub target_id: i32,
}
