use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a projectile is launched.
#[cancellable]
#[derive(Event, Clone)]
pub struct ProjectileLaunchEvent {
    /// The ID of the projectile entity.
    pub entity_id: i32,
    /// The ID of the shooter entity if applicable.
    pub shooter_id: Option<i32>,
}

impl ProjectileLaunchEvent {
    #[must_use]
    pub const fn new(entity_id: i32, shooter_id: Option<i32>) -> Self {
        Self {
            entity_id,
            shooter_id,
            cancelled: false,
        }
    }
}
