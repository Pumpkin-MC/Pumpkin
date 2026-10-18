use pumpkin_macros::Event;

/// An event that occurs when a map is initialized.
#[derive(Event, Clone)]
pub struct MapInitializeEvent {
    /// The ID of the initialized map.
    pub map_id: i32,
}

impl MapInitializeEvent {
    #[must_use]
    pub const fn new(map_id: i32) -> Self {
        Self { map_id }
    }
}
