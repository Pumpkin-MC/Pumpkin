use pumpkin_macros::{Event, cancellable};

/// An event that occurs when the number of arrows stuck in an entity changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct ArrowBodyCountChangeEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// Old arrow count.
    pub old_amount: u32,
    /// New arrow count.
    pub new_amount: u32,
}

impl ArrowBodyCountChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, old_amount: u32, new_amount: u32) -> Self {
        Self {
            entity_id,
            old_amount,
            new_amount,
            cancelled: false,
        }
    }
}
