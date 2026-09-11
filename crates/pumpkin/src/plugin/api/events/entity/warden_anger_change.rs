use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a warden's anger towards an entity changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct WardenAngerChangeEvent {
    /// The ID of the warden entity.
    pub entity_id: i32,

    /// The ID of the target entity.
    pub target_id: i32,

    /// The previous anger level.
    pub old_anger: i32,

    /// The new anger level.
    pub new_anger: i32,
}
