use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a horse jumps.
#[cancellable]
#[derive(Event, Clone)]
pub struct HorseJumpEvent {
    /// Horse entity ID.
    pub entity_id: i32,
    /// Jump power.
    pub power: f32,
}

impl HorseJumpEvent {
    #[must_use]
    pub const fn new(entity_id: i32, power: f32) -> Self {
        Self {
            entity_id,
            power,
            cancelled: false,
        }
    }
}
