use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a sheep has its wool dyed.
#[cancellable]
#[derive(Event, Clone)]
pub struct SheepDyeWoolEvent {
    /// The ID of the sheep.
    pub entity_id: i32,

    /// The new dye color index.
    pub dye_color: u8,

    /// The player ID who dyed the sheep, if any.
    pub player_id: Option<i32>,
}
