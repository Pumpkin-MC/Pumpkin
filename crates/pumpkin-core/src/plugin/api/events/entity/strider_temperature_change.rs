use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a strider's shivering status changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct StriderTemperatureChangeEvent {
    /// The ID of the strider entity.
    pub entity_id: i32,

    /// Whether the strider is now shivering.
    pub is_shivering: bool,
}

impl StriderTemperatureChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_shivering: bool) -> Self {
        Self {
            entity_id,
            is_shivering,
            cancelled: false,
        }
    }
}
