use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a firework rocket explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct FireworkExplodeEvent {
    /// The ID of the firework entity.
    pub entity_id: i32,
}
