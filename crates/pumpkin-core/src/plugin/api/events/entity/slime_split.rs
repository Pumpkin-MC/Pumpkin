use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a slime splits into smaller slimes.
#[cancellable]
#[derive(Event, Clone)]
pub struct SlimeSplitEvent {
    /// The ID of the parent slime entity.
    pub entity_id: i32,

    /// The number of smaller slimes created.
    pub count: i32,
}
