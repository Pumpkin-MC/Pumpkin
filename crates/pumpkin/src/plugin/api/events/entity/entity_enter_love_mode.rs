use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an animal enters love mode for breeding.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityEnterLoveModeEvent {
    /// The ID of the animal entering love mode.
    pub entity_id: i32,

    /// The ID of the human player that fed the animal, if any.
    pub human_entity_id: Option<i32>,

    /// The duration of love mode in ticks.
    pub ticks_in_love: i32,
}

impl EntityEnterLoveModeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, human_entity_id: Option<i32>, ticks_in_love: i32) -> Self {
        Self {
            entity_id,
            human_entity_id,
            ticks_in_love,
            cancelled: false,
        }
    }
}
