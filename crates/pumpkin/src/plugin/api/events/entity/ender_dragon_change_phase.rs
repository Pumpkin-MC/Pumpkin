use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an Ender Dragon changes its phase.
#[cancellable]
#[derive(Event, Clone)]
pub struct EnderDragonChangePhaseEvent {
    /// The ID of the Ender Dragon.
    pub entity_id: i32,

    /// The current dragon phase.
    pub current_phase: String,

    /// The new dragon phase.
    pub new_phase: String,
}
