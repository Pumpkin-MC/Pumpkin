use pumpkin_macros::{Event, cancellable};

/// An event that occurs when two entities breed to create a child.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityBreedEvent {
    /// The ID of the parent 1 entity.
    pub father_id: i32,

    /// The ID of the parent 2 entity.
    pub mother_id: i32,

    /// The ID of the child entity.
    pub child_id: i32,
}

impl EntityBreedEvent {
    #[must_use]
    pub const fn new(father_id: i32, mother_id: i32, child_id: i32) -> Self {
        Self {
            father_id,
            mother_id,
            child_id,
            cancelled: false,
        }
    }
}
