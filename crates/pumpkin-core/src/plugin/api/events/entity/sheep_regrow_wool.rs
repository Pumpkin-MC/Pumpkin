use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a sheep regrows its wool.
#[cancellable]
#[derive(Event, Clone)]
pub struct SheepRegrowWoolEvent {
    /// The ID of the sheep entity.
    pub entity_id: i32,
}
