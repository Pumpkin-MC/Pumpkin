use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a tamed animal is ordered to sit or to stand up.
///
/// Cancelling keeps the previous order.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityToggleSitEvent {
    /// The ID of the animal.
    pub entity_id: i32,

    /// Whether the animal is now ordered to sit.
    pub sitting: bool,
}

impl EntityToggleSitEvent {
    #[must_use]
    pub const fn new(entity_id: i32, sitting: bool) -> Self {
        Self {
            entity_id,
            sitting,
            cancelled: false,
        }
    }
}
