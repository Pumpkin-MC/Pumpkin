use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a spellcasting entity casts a spell.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntitySpellCastEvent {
    /// The ID of the spellcaster entity.
    pub entity_id: i32,

    /// The spell name being cast.
    pub spell: String,
}
