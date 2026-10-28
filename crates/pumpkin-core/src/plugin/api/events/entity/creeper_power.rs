use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a creeper is powered.
#[cancellable]
#[derive(Event, Clone)]
pub struct CreeperPowerEvent {
    /// Creeper entity ID.
    pub entity_id: i32,
    /// Lightning bolt entity ID if caused by lightning.
    pub lightning_id: Option<i32>,
    /// Power cause.
    pub cause: String,
}

impl CreeperPowerEvent {
    #[must_use]
    pub const fn new(entity_id: i32, lightning_id: Option<i32>, cause: String) -> Self {
        Self {
            entity_id,
            lightning_id,
            cause,
            cancelled: false,
        }
    }
}
